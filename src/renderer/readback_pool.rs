use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ReadbackEntry {
    pub capacity: u64,
    pub buffer: Arc<wgpu::Buffer>,
}

#[derive(Debug)]
/// A small LRU pool for readback buffers keyed by capacity
///
/// This pool is kept alive across async map window; cannot be trivially suballocated
/// like UNIFORM slices due to MAP_READ semantics and row padding.
///
///  •  Purpose: Readback pixels (COPY_DST + MAP_READ) without re-allocating a fresh buffer each time
///  •  LRU of whole buffers keyed by capacity; best-fit selection; Returns as Arc
///  •  Usage: COPY_DST | MAP_READ; mapped for read after GPU completes.
pub(crate) struct ReadbackBufferPool {
    label: String,
    entry_limit: usize,
    entries: VecDeque<ReadbackEntry>, // front = LRU, back = MRU
}

impl ReadbackBufferPool {
    pub fn new(label: &str, entry_limit: usize) -> Self {
        Self {
            label: label.to_string(),
            entry_limit: entry_limit.max(1),
            entries: VecDeque::new(),
        }
    }

    /// Get a buffer with at least `size` bytes of capacity.
    /// Returns an Arc handle so callers can hold it without borrowing the pool.
    pub fn get(&mut self, device: &wgpu::Device, size: u64) -> Arc<wgpu::Buffer> {
        // Find the smallest buffer that fits to reduce waste
        let mut best_index: Option<usize> = None;
        let mut best_capacity: u64 = u64::MAX;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.capacity >= size && entry.capacity < best_capacity {
                best_capacity = entry.capacity;
                best_index = Some(i);
            }
        }

        if let Some(idx) = best_index {
            // Move to MRU position and return
            let entry = self.entries.remove(idx).expect("valid index");
            let buffer = entry.buffer.clone();
            self.entries.push_back(entry);
            return buffer;
        }

        // No suitable entry; create a new buffer exactly sized to `size`
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let arc = Arc::new(buffer);
        self.entries.push_back(ReadbackEntry {
            capacity: size,
            buffer: arc.clone(),
        });

        // Evict LRU if over capacity
        if self.entries.len() > self.entry_limit {
            let _ = self.entries.pop_front();
        }
        arc
    }
}

#[cfg(test)]
mod tests {
    use super::readback_pool::ReadbackBufferPool;
    use super::*;
    use std::sync::Arc;

    // Helper to get a test device
    async fn device() -> wgpu::Device {
        let instance = crate::renderer::platform::all::create_instance().await;
        let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, _queue) = crate::renderer::platform::all::request_device(&adapter)
            .await
            .expect("device");
        device
    }

    #[test]
    fn pool_reuses_and_best_fits() {
        pollster::block_on(async move {
            let device = device().await;
            let mut pool = ReadbackBufferPool::new("Test Readback Pool", 8);

            // Request 1024 bytes, then 1536 should pick a new or larger buffer
            let b1 = pool.get(&device, 1024);
            let b2 = pool.get(&device, 1536);
            assert!(!Arc::ptr_eq(&b1, &b2));

            // Request 1100 should best-fit to b2 (1536) rather than creating new
            let b3 = pool.get(&device, 1100);
            assert!(Arc::ptr_eq(&b2, &b3));

            // Request 900 should best-fit to b1 (1024)
            let b4 = pool.get(&device, 900);
            assert!(Arc::ptr_eq(&b1, &b4));
        });
    }

    #[test]
    fn pool_evicts_lru() {
        pollster::block_on(async move {
            let device = device().await;
            // Small limit to force eviction
            let mut pool = ReadbackBufferPool::new("Test Readback Pool", 2);

            let b1 = pool.get(&device, 512);
            let b2 = pool.get(&device, 1024);
            // Access b1 to make it MRU, b2 becomes LRU
            let _ = pool.get(&device, 256); // best fits to b1

            // Insert third; should evict b2 (old LRU)
            let b3 = pool.get(&device, 2048);
            assert!(!Arc::ptr_eq(&b2, &b3));

            // Now a request that best-fits 1024 should not find b2 and will allocate or reuse b3
            let b4 = pool.get(&device, 1024);
            // Either equals b3 (if capacity fits best) or a new allocation; must not be b2
            assert!(!Arc::ptr_eq(&b4, &b2));

            // Silence warnings
            let _ = b1;
        });
    }
}
