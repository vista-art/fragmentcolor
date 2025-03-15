use std::num::NonZeroU64;

/// Default chunk size for buffer allocation (64KB)
const DEFAULT_CHUNK_SIZE: u64 = 0x10000;

#[derive(Debug)]
/// A pool of GPU buffers that manages allocations in fixed-size chunks
pub(crate) struct BufferPool {
    label: String,
    usage: wgpu::BufferUsages,
    buffers: Vec<wgpu::Buffer>,
    chunk_size: u64,
    current_chunk: usize,
    current_offset: u64,
    pub alignment: u64,
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
            device,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            DEFAULT_CHUNK_SIZE,
        )
    }

    // TODO add more buffer pool types

    /// Creates a new buffer pool with custom parameters
    pub fn new(
        label: &str,
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        chunk_size: u64,
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
            alignment: device.limits().min_uniform_buffer_offset_alignment as u64,
        }
    }

    /// Ensures the pool has enough capacity for the total required size.
    ///
    /// Must be called before upload, normally at the beginning of a frame.
    pub fn ensure_capacity(&mut self, required_bytes: u64, device: &wgpu::Device) {
        let remaining_in_current = self.chunk_size - self.current_offset;
        let chunks_after_current =
            (self.buffers.len() as u64).saturating_sub(self.current_chunk as u64 + 1);
        let available = remaining_in_current + (chunks_after_current * self.chunk_size);

        if available >= required_bytes {
            return;
        }

        let needed_chunks =
            (required_bytes - available).saturating_add(self.chunk_size - 1) / self.chunk_size;
        for _ in 0..needed_chunks {
            self.buffers
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&self.label),
                    size: self.chunk_size,
                    usage: self.usage,
                    mapped_at_creation: false,
                }));
        }
    }

    /// Upload raw bytes to the pool, returns buffer location
    pub fn upload(&mut self, data: &[u8], queue: &wgpu::Queue) -> BufferLocation {
        let size = data.len() as u64;
        let aligned_size = wgpu::util::align_to(size, self.alignment);

        assert!(
            aligned_size <= self.chunk_size,
            "Data chunk too large for buffer pool"
        );

        // Advance to next chunk if needed
        if self.current_offset + aligned_size > self.chunk_size {
            self.current_chunk += 1;
            self.current_offset = 0;
            assert!(
                self.current_chunk < self.buffers.len(),
                "Buffer pool overflow - call ensure_capacity first"
            );
        }

        // Write to current chunk
        queue.write_buffer(&self.buffers[self.current_chunk], self.current_offset, data);

        let location = BufferLocation {
            chunk_index: self.current_chunk,
            offset: self.current_offset,
            size,
        };

        self.current_offset += aligned_size;
        location
    }

    /// Gets a buffer binding suitable for use in a bind group
    pub fn get_binding(&self, location: BufferLocation) -> wgpu::BufferBinding {
        wgpu::BufferBinding {
            buffer: &self.buffers[location.chunk_index],
            offset: location.offset,
            size: match location.size {
                0 => None,
                _ => Some(NonZeroU64::new(location.size).unwrap()),
            },
        }
    }

    /// Resets the pool for reuse in the next frame.
    ///
    /// Must be called at the start or the end of every frame.
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.current_offset = 0;
    }
}
