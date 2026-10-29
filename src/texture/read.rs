use std::sync::Arc;

use crate::renderer::Start;
use crate::texture::{Texture, TextureError, TextureObject};
use crate::{PendingReadback, RenderContext};

struct ReadbackPlan {
    buffer: Arc<wgpu::Buffer>,
    row_bytes: u32,
    padded_row_bytes: u32,
    width: u32,
    height: u32,
    depth: u32,
    bpp: u32,
}

fn prepare_readback(
    context: &RenderContext,
    texture: &TextureObject,
    label: &'static str,
) -> Result<ReadbackPlan, TextureError> {
    if !texture.usage.contains(wgpu::TextureUsages::COPY_SRC) {
        return Err(TextureError::Error(
            "Texture usage does not allow readback (missing COPY_SRC)".into(),
        ));
    }

    let format = texture.format;
    let flags = context
        .adapter
        .as_ref()
        .map_or(wgpu::DownlevelFlags::all(), |adapter| {
            adapter.get_downlevel_capabilities().flags
        });
    check_copy(format, flags)?;
    let bpp = super::texel_size(format)?;

    let size = texture.size;
    let width = size.width;
    let height = size.height;
    let depth = size.depth_or_array_layers.max(1);

    if width == 0 || height == 0 {
        return Err(TextureError::Error(
            "Texture extent must be non-zero in width and height".into(),
        ));
    }

    let row_bytes = width.saturating_mul(bpp);
    let padded_row_bytes =
        wgpu::util::align_to(row_bytes as u64, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64) as u32;
    let output_size = padded_row_bytes as u64 * height as u64 * depth as u64;
    check_buffer(output_size, context.device.limits().max_buffer_size, size)?;

    let buffer = {
        let mut pool = context.readback_pool.write();
        pool.get(&context.device, output_size)
    };

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture.inner,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: Some(height),
            },
        },
        size,
    );

    context.queue.submit(Some(encoder.finish()));

    Ok(ReadbackPlan {
        buffer,
        row_bytes,
        padded_row_bytes,
        width,
        height,
        depth,
        bpp,
    })
}

/// Checks that a backend copies a texture of `format` to a buffer.
/// `flags` are its downlevel flags, and GLES and WebGL2 copy no depth texture.
fn check_copy(
    format: wgpu::TextureFormat,
    flags: wgpu::DownlevelFlags,
) -> Result<(), TextureError> {
    let copies = flags.contains(wgpu::DownlevelFlags::DEPTH_TEXTURE_AND_BUFFER_COPIES);
    if format.is_depth_stencil_format() && !copies {
        return Err(TextureError::Readback(format!(
            "this backend can't copy a {format:?} texture to a buffer"
        )));
    }
    Ok(())
}

/// Checks that a readback buffer of `bytes` for a texture of `size` fits the device's `limit`.
fn check_buffer(bytes: u64, limit: u64, size: wgpu::Extent3d) -> Result<(), TextureError> {
    if bytes <= limit {
        return Ok(());
    }
    let wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: depth,
    } = size;
    Err(TextureError::Readback(format!(
        "can't read back a {width}x{height}x{depth} texture, the copy needs {bytes} bytes and the device allows {limit}"
    )))
}

fn extract_pixels(plan: &ReadbackPlan) -> Result<Vec<u8>, TextureError> {
    let slice = plan.buffer.slice(..);
    let view = slice.get_mapped_range().map_err(|e| {
        TextureError::Error(format!("texture readback get_mapped_range failed: {e:?}"))
    })?;
    let layer_stride = plan.padded_row_bytes as usize * plan.height as usize;
    let mut pixels = Vec::with_capacity(
        plan.width as usize * plan.height as usize * plan.depth as usize * plan.bpp as usize,
    );
    for z in 0..plan.depth as usize {
        for y in 0..plan.height as usize {
            let start = z * layer_stride + y * plan.padded_row_bytes as usize;
            pixels.extend_from_slice(&view[start..start + plan.row_bytes as usize]);
        }
    }
    drop(view);
    plan.buffer.unmap();
    Ok(pixels)
}

/// Read the mip-0 contents of `texture` as tightly-packed bytes in the texture's
/// native format, once the frames rendered before the call have reported their GPU errors.
pub(crate) async fn read_pixels(
    context: &RenderContext,
    texture: &TextureObject,
) -> Result<Vec<u8>, TextureError> {
    read_pixels_from(context, texture, Start::now(Some(context))).await
}

/// Reads like `read_pixels`, once the frames rendered before `start` have reported their GPU errors.
pub(crate) async fn read_pixels_from(
    context: &RenderContext,
    texture: &TextureObject,
    start: Start,
) -> Result<Vec<u8>, TextureError> {
    #[cfg(wasm)]
    start.settle(context).await?;
    #[cfg(not(wasm))]
    let _ = start;
    copy_pixels(context, texture).await
}

/// Copies the mip-0 contents of `texture` to the CPU, for a caller that settled the late errors.
///
/// The map callback fires only from a device poll. On native `wait_for_map` polls the device
/// until it fires and returns `TextureError::Readback` after `MAP_LIMIT`, so the readback ends
/// whether or not other code polls the device. On web, WebGPU schedules the callback itself,
/// while the WebGL2 backend only fires it from a poll, so `await_map_on_web` polls between turns.
pub(crate) async fn copy_pixels(
    context: &RenderContext,
    texture: &TextureObject,
) -> Result<Vec<u8>, TextureError> {
    let plan = prepare_readback(context, texture, "Texture readback encoder")?;
    let readback = PendingReadback::new(&context.readback_pool, plan.buffer.clone());

    let slice = plan.buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });

    #[cfg(not(wasm))]
    {
        let mut rx = rx;
        crate::wait_for_map(&context.device, &mut rx, crate::MAP_LIMIT)
            .map_err(|error| TextureError::Readback(error.to_string()))?;
    }

    #[cfg(wasm)]
    if !await_map_on_web(context, rx).await {
        return Err(TextureError::Readback(
            "mapping did not complete before the timeout".into(),
        ));
    }

    let pixels = extract_pixels(&plan)?;
    readback.finish();
    Ok(pixels)
}

pub(super) async fn get_image(texture: &Texture) -> Result<Vec<u8>, TextureError> {
    read_pixels(&texture.context, &texture.object()).await
}

/// Drives the device until the map callback fires. WebGPU resolves the map
/// on its own, but the WebGL2 backend only signals it from a poll after its
/// fence completes, so the loop yields to the event loop between polls.
#[cfg(wasm)]
pub(crate) async fn await_map_on_web(context: &RenderContext, mut rx: crate::MapReceiver) -> bool {
    // ~10 s of zero-delay timers before giving up
    for _ in 0..2000 {
        let _ = context.device.poll(wgpu::PollType::Poll);
        match rx.try_recv() {
            Ok(Some(Ok(()))) => return true,
            Ok(Some(Err(_))) | Err(_) => return false,
            Ok(None) => crate::platforms::web::yield_to_event_loop().await,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: a backend without depth copies, such as GLES or WebGL2, refuses to
    // read a depth texture back, and still reads color textures. Pure, no GPU.
    #[test]
    fn check_copy_refuses_depth_without_depth_copies() {
        let depth = wgpu::TextureFormat::Depth32Float;
        let color = wgpu::TextureFormat::Rgba8Unorm;
        let gles =
            wgpu::DownlevelFlags::all() - wgpu::DownlevelFlags::DEPTH_TEXTURE_AND_BUFFER_COPIES;
        assert!(matches!(
            check_copy(depth, gles),
            Err(TextureError::Readback(_))
        ));
        check_copy(color, gles).expect("color copies everywhere");
        check_copy(depth, wgpu::DownlevelFlags::all()).expect("depth copies on WebGPU");
    }

    // Story: a readback whose padded buffer is past the device's largest buffer
    // is refused before the buffer is made, and one that fits passes. Pure, no GPU.
    #[test]
    fn check_buffer_refuses_a_copy_past_the_limit() {
        let size = wgpu::Extent3d {
            width: 16384,
            height: 16384,
            depth_or_array_layers: 1,
        };
        let limit = 256 << 20;
        let refused = check_buffer(16384 * 16 * 16384, limit, size);
        assert!(matches!(refused, Err(TextureError::Readback(_))));
        check_buffer(limit, limit, size).expect("a copy at the limit fits");
    }
}
