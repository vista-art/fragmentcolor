//! Reads a shader's storage buffer back to the CPU through a pooled readback buffer.

#[cfg(not(wasm))]
use crate::core::renderer::{MAP_LIMIT, poll_map};
use crate::core::renderer::{PendingReadback, RenderContext, RendererError};

/// Copy `[0, span)` of `buffer` into a pooled readback buffer, map it,
/// extract the bytes, and return them.
///
/// Caller has already validated:
///   - the shader declares the storage binding,
///   - the renderer's `storage_registry` has materialized the GPU buffer
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
    let readback = PendingReadback::new(&context.readback_pool, staging.clone());

    let slice = staging.slice(0..span);
    let (sender, receiver) = futures::channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });

    #[cfg(not(wasm))]
    {
        let mut receiver = receiver;
        poll_map(&context.device, &mut receiver, MAP_LIMIT)?;
    }

    #[cfg(wasm)]
    if !crate::texture::read::await_map_on_web(context, receiver).await {
        return Err(RendererError::Error(
            "storage readback mapping did not complete".into(),
        ));
    }

    let view = slice.get_mapped_range().map_err(|e| {
        RendererError::Error(format!("storage readback get_mapped_range failed: {e:?}"))
    })?;
    let bytes = view.to_vec();
    drop(view);
    staging.unmap();
    readback.finish();
    Ok(bytes)
}
