use std::num::NonZeroU64;

/// Default chunk size for buffer allocation (64KB)
pub(super) const DEFAULT_CHUNK_SIZE: u64 = 0x10000;

#[derive(Debug)]
/// A pool of GPU buffers that manages allocations in fixed-size chunks
///
///  •  Purpose: Upload uniforms in-frame with alignment padding (typically 256)
///  •  Grows by fixed-size chunks; suballocates many small ranges per frame; reset between frames
///  •  Usage: UNIFORM | COPY_DST; not mapped for read
pub(crate) struct UniformBufferPool {
    label: String,
    usage: wgpu::BufferUsages,
    buffers: Vec<wgpu::Buffer>,
    chunk_size: u64,
    current_chunk: usize,
    current_offset: u64,
    pub alignment: u64,
    // metrics
    allocations: u64,
    bytes_allocated: u64,
}

/// Represents a location within a BufferPool
#[derive(Debug, Clone, Copy)]
pub(crate) struct BufferLocation {
    pub chunk_index: usize,
    pub offset: u64,
    pub size: u64,
}

impl UniformBufferPool {
    /// Creates a new Uniform Buffer Pool that can be used as a destination buffer for:
    ///
    /// - CommandEncoder::copy_buffer_to_buffer,
    /// - CommandEncoder::copy_texture_to_buffer,
    /// - CommandEncoder::clear_buffer or
    /// - Queue::write_buffer
    pub fn new(label: &str, device: &wgpu::Device) -> Self {
        Self::with_params(
            label,
            device,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            DEFAULT_CHUNK_SIZE,
        )
    }

    /// Creates a new buffer pool with custom parameters
    pub fn with_params(
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
            allocations: 1,
            bytes_allocated: chunk_size,
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
            self.allocations += 1;
            self.bytes_allocated += self.chunk_size;
        }
    }

    /// Upload raw bytes to the pool, returns buffer location
    pub fn upload(
        &mut self,
        data: &[u8],
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> BufferLocation {
        let size = data.len() as u64;
        let aligned_size = wgpu::util::align_to(size, self.alignment);

        // If a single upload does not fit in current chunk size, allocate a dedicated buffer
        if aligned_size > self.chunk_size {
            let idx = {
                self.buffers
                    .push(device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&self.label),
                        size: aligned_size,
                        usage: self.usage,
                        mapped_at_creation: false,
                    }));
                self.allocations += 1;
                self.bytes_allocated += aligned_size;
                self.buffers.len() - 1
            };

            // Write directly to the dedicated buffer at offset 0
            queue.write_buffer(&self.buffers[idx], 0, data);

            // Update cursor to the end of this dedicated buffer so subsequent writes advance
            self.current_chunk = idx;
            self.current_offset = aligned_size;

            return BufferLocation {
                chunk_index: idx,
                offset: 0,
                size,
            };
        }

        // Advance to next chunk if the current one doesn't have enough space
        if self.current_offset + aligned_size > self.chunk_size {
            self.current_chunk += 1;
            self.current_offset = 0;
            // Grow lazily if needed
            if self.current_chunk >= self.buffers.len() {
                self.buffers
                    .push(device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&self.label),
                        size: self.chunk_size,
                        usage: self.usage,
                        mapped_at_creation: false,
                    }));
                self.allocations += 1;
                self.bytes_allocated += self.chunk_size;
            }
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
    pub fn get_binding(&self, location: BufferLocation) -> wgpu::BufferBinding<'_> {
        // Pad binding size to the device's uniform buffer alignment (often 256 bytes) to
        // avoid backend quirks. Ensure a minimum of 16 bytes as a safety floor.
        let padded_size = if location.size == 0 {
            0
        } else {
            let align = self.alignment.max(16);
            wgpu::util::align_to(location.size, align)
        };

        wgpu::BufferBinding {
            buffer: &self.buffers[location.chunk_index],
            offset: location.offset,
            size: match padded_size {
                0 => None,
                _ => Some(unsafe { NonZeroU64::new_unchecked(padded_size) }),
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

impl crate::renderer::buffer_pool::BufferPool for UniformBufferPool {
    fn stats(&self) -> crate::renderer::buffer_pool::PoolStats {
        crate::renderer::buffer_pool::PoolStats {
            hits: 0,
            misses: 0,
            evictions: 0,
            allocations: self.allocations,
            bytes_allocated: self.bytes_allocated,
        }
    }

    fn reset_metrics(&mut self) {
        self.allocations = 1; // initial buffer accounted
        self.bytes_allocated = self.chunk_size;
    }
}

#[cfg(test)]
#[cfg(not(wasm))]
mod tests {
    use super::*;
    use crate::renderer::buffer_pool::BufferPool;

    async fn device_and_queue() -> (wgpu::Device, wgpu::Queue) {
        let instance = crate::renderer::platform::all::create_instance().await;
        let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
            .await
            .expect("adapter");
        crate::renderer::platform::all::request_device(&adapter)
            .await
            .expect("device")
    }

    // Story: ensure_capacity grows the pool by whole chunks when required capacity exceeds current slack.
    #[test]
    fn grows_capacity_in_chunks() {
        pollster::block_on(async move {
            // Arrange
            let (device, _queue) = device_and_queue().await;
            let mut pool = UniformBufferPool::with_params(
                "test pool",
                &device,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                4096,
            );
            let initial = pool.stats();

            // Act: require 3 chunks worth
            pool.ensure_capacity(3 * 4096 + 1, &device);

            // Assert
            let after = pool.stats();
            assert!(after.bytes_allocated >= initial.bytes_allocated + 2 * 4096);
            assert!(after.allocations >= initial.allocations + 2);
        });
    }

    // Story: upload aligns writes, bindings report sizes padded to at least 16 bytes.
    #[test]
    fn upload_alignment_and_binding_padding() {
        pollster::block_on(async move {
            // Arrange
            let (device, queue) = device_and_queue().await;
            let mut pool = UniformBufferPool::new("pad pool", &device);
            let small = [1u8, 2, 3, 4, 5];

            // Act
            let loc = pool.upload(&small, &queue, &device);
            let binding = pool.get_binding(loc);

            // Assert: binding size is present and padded to multiple of 16
            let sz = binding.size.map(|nz| nz.get()).unwrap_or(0);
            assert!(sz >= 16 && sz.is_multiple_of(16));

            // Zero-sized case yields None
            let zero: [u8; 0] = [];
            let loc0 = pool.upload(&zero, &queue, &device);
            let b0 = pool.get_binding(loc0);
            assert!(b0.size.is_none());
        });
    }

    // Story: reset sets the write cursor back to the beginning so the next upload starts at offset 0.
    #[test]
    fn reset_rewinds_cursor() {
        pollster::block_on(async move {
            // Arrange
            let (device, queue) = device_and_queue().await;
            let mut pool = UniformBufferPool::with_params(
                "rewind pool",
                &device,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                1024,
            );

            // Act: write some data then reset
            let loc1 = pool.upload(&[0u8; 32], &queue, &device);
            assert!(loc1.offset == 0);
            let _ = pool.upload(&[0u8; 32], &queue, &device);
            pool.reset();
            let loc2 = pool.upload(&[0u8; 16], &queue, &device);

            // Assert: after reset, offset is 0 again
            assert_eq!(loc2.offset, 0);
        });
    }
}
