use encase::{internal::WriteInto, ShaderType};
use std::num::NonZeroU64;

/// Default chunk size for buffer allocation (64KB)
const DEFAULT_CHUNK_SIZE: u64 = 0x10000;

/// A pool of GPU buffers that manages allocations in fixed-size chunks
pub struct BufferPool {
    label: String,
    usage: wgpu::BufferUsages,
    buffers: Vec<wgpu::Buffer>,
    chunk_size: u64,
    current_chunk: usize,
    current_offset: u64,
    alignment: u64,
}

/// Represents a location within a BufferPool
#[derive(Debug, Clone, Copy)]
pub struct BufferLocation {
    pub chunk_index: usize,
    pub offset: u64,
    pub size: u64,
}

impl BufferPool {
    /// Creates a new Uniform Buffer Pool
    /// that can be used as a destination buffer for:
    /// - CommandEncoder::copy_buffer_to_buffer, 
    /// - CommandEncoder::copy_texture_to_buffer, 
    /// - CommandEncoder::clear_buffer or 
    /// - Queue::write_buffer 
    pub fn new_uniform_pool(label: &str, device: &wgpu::Device) -> Self {
        Self::new(
            label,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            device.limits().min_uniform_buffer_offset_alignment as u64,
            DEFAULT_CHUNK_SIZE,
            device,
        )
    }

    /// Creates a new buffer pool with custom parameters
    pub fn new(
        label: &str,
        usage: wgpu::BufferUsages,
        alignment: u64,
        chunk_size: u64,
        device: &wgpu::Device,
    ) -> Self {
        let initial_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: chunk_size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            label: label.to_string(),
            usage,
            buffers: vec![initial_buffer],
            chunk_size,
            current_chunk: 0,
            current_offset: 0,
            alignment,
        }
    }

    /// Ensures the pool has enough capacity for the total required size
    pub fn ensure_capacity<T: ShaderType>(
        &mut self,
        required_size: usize,
        device: &wgpu::Device,
    ) -> usize {
        if required_size == 0 {
            return 0;
        }

        let total_size = self.calculate_storage_size::<T>(required_size);
        let needed_chunks = 1 + ((total_size - 1) / self.chunk_size as usize);

        while self.buffers.len() < needed_chunks {
            self.buffers
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&self.label),
                    size: self.chunk_size,
                    usage: self.usage,
                    mapped_at_creation: false,
                }));
        }

        needed_chunks
    }

    /// Allocates space for and uploads data using encase for serialization
    pub fn upload<T>(&mut self, value: &T, queue: &wgpu::Queue) -> BufferLocation
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(value).unwrap();
        let data = buffer.into_inner();
        let size = data.len() as u64;

        let aligned_size = if self.alignment > 0 {
            (size + self.alignment - 1) & !(self.alignment - 1)
        } else {
            size
        };

        assert!(
            aligned_size <= self.chunk_size,
            "Object too large for chunk size"
        );

        if self.current_offset + aligned_size > self.chunk_size {
            self.current_chunk += 1;
            self.current_offset = 0;

            assert!(
                self.current_chunk < self.buffers.len(),
                "Buffer pool ran out of chunks - call ensure_capacity first"
            );
        }

        let location = BufferLocation {
            chunk_index: self.current_chunk,
            offset: self.current_offset,
            size,
        };

        queue.write_buffer(
            &self.buffers[self.current_chunk],
            self.current_offset,
            &data,
        );

        self.current_offset += aligned_size;

        location
    }

    /// Gets a buffer binding suitable for use in a bind group
    pub fn get_binding<T: ShaderType>(&self, location: BufferLocation) -> wgpu::BufferBinding {
        wgpu::BufferBinding {
            buffer: &self.buffers[location.chunk_index],
            offset: location.offset,
            size: match location.size {
                0 => None,
                _ => Some(NonZeroU64::new(location.size).unwrap()),
            },
        }
    }

    /// Resets the pool for reuse in the next frame
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.current_offset = 0;
    }
}

impl BufferPool {
    /// Calculates required storage for a given type
    fn calculate_storage_size<T: ShaderType>(&self, count: usize) -> usize {
        let size = T::min_size();
        let aligned_size = if self.alignment > 0 {
            ((size.get() + self.alignment - 1) & !(self.alignment - 1)) as usize
        } else {
            size.get() as usize
        };
        aligned_size * count
    }
}
