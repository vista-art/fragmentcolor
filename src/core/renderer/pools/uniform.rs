//! `UniformBufferPool`: uniform buffers cut into fixed chunks and rewound once per frame.

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
    /// Buffers for uploads past a chunk. The n-th such upload of a render takes the
    /// n-th buffer, which grows only when the upload passes its size.
    dedicated: Vec<wgpu::Buffer>,
    next_dedicated: usize,
    chunk_size: u64,
    current_chunk: usize,
    current_offset: u64,
    pub alignment: u64,
}

/// A location within the buffers of a pool.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BufferLocation {
    pub chunk_index: usize,
    pub offset: u64,
    pub size: u64,
    /// Whether `chunk_index` names one of the buffers for uploads past a chunk.
    pub dedicated: bool,
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
            dedicated: Vec::new(),
            next_dedicated: 0,
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
    pub fn upload(
        &mut self,
        data: &[u8],
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) -> BufferLocation {
        let size = data.len() as u64;
        let aligned_size = wgpu::util::align_to(size, self.alignment);

        // An upload past a chunk takes a buffer of its own, kept for the next render.
        if aligned_size > self.chunk_size {
            let index = self.next_dedicated;
            self.next_dedicated += 1;
            let fits = self
                .dedicated
                .get(index)
                .is_some_and(|buffer| buffer.size() >= aligned_size);
            if !fits {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&self.label),
                    size: aligned_size,
                    usage: self.usage,
                    mapped_at_creation: false,
                });
                match self.dedicated.get_mut(index) {
                    Some(slot) => *slot = buffer,
                    None => self.dedicated.push(buffer),
                }
            }
            queue.write_buffer(&self.dedicated[index], 0, data);
            return BufferLocation {
                chunk_index: index,
                offset: 0,
                size,
                dedicated: true,
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
            }
        }

        // Write to current chunk
        queue.write_buffer(&self.buffers[self.current_chunk], self.current_offset, data);

        let location = BufferLocation {
            chunk_index: self.current_chunk,
            offset: self.current_offset,
            size,
            dedicated: false,
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

        let buffers = match location.dedicated {
            true => &self.dedicated,
            false => &self.buffers,
        };
        wgpu::BufferBinding {
            buffer: &buffers[location.chunk_index],
            offset: location.offset,
            size: match padded_size {
                0 => None,
                _ => Some(unsafe { NonZeroU64::new_unchecked(padded_size) }),
            },
        }
    }

    /// Rewinds the write cursor to the start of the pool.
    ///
    /// Call it once per render, before any pass uploads.
    /// Uploads apply at the next submit, so a rewind between
    /// passes of one submit lets later passes overwrite earlier ones.
    pub fn reset(&mut self) {
        self.current_chunk = 0;
        self.current_offset = 0;
        self.next_dedicated = 0;
    }
}

#[cfg(test)]
#[cfg(not(wasm))]
mod tests {
    use super::*;

    async fn device_and_queue() -> (wgpu::Device, wgpu::Queue) {
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        crate::core::renderer::device::request_device(&adapter)
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
            let initial = pool.buffers.len();

            // Act: require 3 chunks worth
            pool.ensure_capacity(3 * 4096 + 1, &device);

            // Assert: one chunk was free, so three more cover the rest
            assert_eq!(pool.buffers.len(), initial + 3);
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

    // Story: an upload past a chunk reuses its buffer on every render, so a large
    // uniform drawn each frame holds one buffer instead of one per frame.
    #[test]
    fn large_uploads_reuse_their_buffer() {
        pollster::block_on(async move {
            let (device, queue) = device_and_queue().await;
            let mut pool = UniformBufferPool::with_params(
                "large pool",
                &device,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                1024,
            );
            let large = vec![7u8; 4096];
            let chunks = pool.buffers.len();
            let mut first_render = Vec::new();

            for _ in 0..3 {
                pool.reset();
                let first = pool.upload(&large, &queue, &device);
                let second = pool.upload(&large, &queue, &device);
                assert!(first.dedicated && second.dedicated);
                assert_ne!(
                    first.chunk_index, second.chunk_index,
                    "one render keeps two apart"
                );
                if first_render.is_empty() {
                    first_render = pool.dedicated.clone();
                }
            }

            assert_eq!(
                pool.dedicated, first_render,
                "the same two buffers serve every render"
            );
            assert_eq!(pool.buffers.len(), chunks, "no chunk was added");
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
