//! GPU→CPU storage-buffer readback.
//!
//! Storage buffers backing a [`Shader`]'s storage bindings live in the
//! renderer's `storage_registry` (`DashMap<binding_name, (wgpu::Buffer, span)>`).
//! After a compute (or render) pass writes to one, the CPU-side mirror on the
//! Shader is stale — readers must round-trip through the GPU.
//!
//! Mirrors [`super::super::texture::read`] (texture readback) so both readback
//! paths use the same `readback_pool` and `map_async` discipline.

use crate::renderer::{RenderContext, RendererError};

/// Copy `[0, span)` of `buffer` into a pooled readback buffer, map it,
/// extract the bytes, and return them.
///
/// Caller has already validated:
///   - the shader declares the storage binding,
///   - the renderer's `storage_registry` has materialised the GPU buffer
///     (i.e. at least one render pass with this binding has fired).
pub(crate) async fn read_buffer_bytes(
    context: &RenderContext,
    buffer: &wgpu::Buffer,
    span: u64,
) -> Result<Vec<u8>, RendererError> {
    if span == 0 {
        return Ok(Vec::new());
    }

    let staging = {
        let mut pool = context.readback_pool.write();
        pool.get(&context.device, span)
    };

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Storage readback encoder"),
        });
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, span);
    context.queue.submit(Some(encoder.finish()));

    let slice = staging.slice(0..span);
    let (tx, rx) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });

    #[cfg(not(wasm))]
    if let Err(e) = context.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(5)),
    }) {
        log::error!("Device poll error during storage readback mapping: {:?}", e);
        return Err(RendererError::Error(format!("storage readback poll: {e:?}")));
    }

    match rx.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return Err(RendererError::Error(format!(
                "storage readback map_async failed: {e:?}"
            )));
        }
        Err(_) => {
            return Err(RendererError::Error(
                "storage readback map_async channel dropped".into(),
            ));
        }
    }

    let view = slice.get_mapped_range();
    let bytes = view.to_vec();
    drop(view);
    staging.unmap();
    Ok(bytes)
}
