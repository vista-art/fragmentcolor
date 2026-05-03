use std::sync::Arc;

use crate::RenderContext;
use crate::texture::{Texture, TextureError, TextureObject};

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
    let bpp = super::bytes_per_pixel(format);
    if bpp == 0 {
        return Err(TextureError::Error(
            "Unsupported format for readback (bytes-per-pixel is 0)".into(),
        ));
    }

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

fn extract_pixels(plan: &ReadbackPlan) -> Vec<u8> {
    let slice = plan.buffer.slice(..);
    let view = slice.get_mapped_range();
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
    pixels
}

/// Read the mip-0 contents of `texture` as tightly-packed bytes in the texture's
/// native format. Blocks the current thread until the readback buffer is mapped.
///
/// On WASM the browser thread cannot block, so the poll fails and this returns an
/// empty Vec with a logged error — prefer [`read_texture_object_async`] there.
pub(crate) fn read_texture_object_sync(
    context: &RenderContext,
    texture: &TextureObject,
) -> Result<Vec<u8>, TextureError> {
    let plan = prepare_readback(context, texture, "Texture readback encoder (sync)")?;

    let slice = plan.buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });

    if let Err(e) = context.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(5)),
    }) {
        log::error!("Device poll error during readback mapping: {:?}", e);
        #[cfg(wasm)]
        {
            log::error!("Ensure the page is cross-origin isolated to enable readbacks.");
        }
        return Ok(Vec::new());
    }
    let _ = rx.recv();

    Ok(extract_pixels(&plan))
}

/// Async variant of [`read_texture_object_sync`] that works on both native and WASM.
///
/// On native the caller must still drive the device forward for the map callback to fire;
/// this function does a synchronous `device.poll(Wait)` before awaiting so callers do not
/// have to. On web the browser schedules the callback automatically, so the poll is a
/// no-op / ignored.
pub(crate) async fn read_texture_object_async(
    context: &RenderContext,
    texture: &TextureObject,
) -> Result<Vec<u8>, TextureError> {
    let plan = prepare_readback(context, texture, "Texture readback encoder (async)")?;

    let slice = plan.buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });

    #[cfg(not(wasm))]
    if let Err(e) = context.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(5)),
    }) {
        log::error!("Device poll error during readback mapping: {:?}", e);
        return Ok(Vec::new());
    }

    let _ = rx.await;

    Ok(extract_pixels(&plan))
}

pub(super) async fn get_image_async(texture: &Texture) -> Result<Vec<u8>, TextureError> {
    read_texture_object_async(&texture.context, &texture.object).await
}
