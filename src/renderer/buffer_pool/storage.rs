use crate::renderer::buffer_pool::uniform::DEFAULT_CHUNK_SIZE;
use crate::{BufferLocation, UniformBufferPool};

#[derive(Debug)]
pub(crate) struct StorageBufferPool(UniformBufferPool);

impl StorageBufferPool {
    pub fn new(label: &str, device: &wgpu::Device) -> Self {
        Self(UniformBufferPool::with_params(
            label,
            device,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            DEFAULT_CHUNK_SIZE,
        ))
    }

    pub fn get_binding(&self, location: BufferLocation) -> wgpu::BufferBinding<'_> {
        self.0.get_binding(location)
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}
