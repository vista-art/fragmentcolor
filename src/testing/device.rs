//! Small devices for tests of the device limits.

use crate::core::renderer::WgpuFailure;
use crate::core::renderer::device::{create_instance, request_adapter};

/// A device that takes buffers of at most `limit` bytes, for tests of the buffer limit.
pub(crate) async fn request_small_device(
    limit: u64,
) -> Result<(wgpu::Device, wgpu::Queue), WgpuFailure> {
    let instance = create_instance().await;
    let adapter = request_adapter(&instance, None).await?;
    let adapter_limits = adapter.limits();
    let limits = wgpu::Limits {
        max_buffer_size: limit,
        max_uniform_buffer_binding_size: adapter_limits.max_uniform_buffer_binding_size.min(limit),
        max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size.min(limit),
        ..adapter_limits
    };
    let device = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: limits,
            ..Default::default()
        })
        .await?;
    Ok(device)
}
