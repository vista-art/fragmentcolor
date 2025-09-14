use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ReadbackEntry {
    pub capacity: u64,
    pub buffer: Arc<wgpu::Buffer>,
}

#[derive(Debug)]
/// A small LRU pool for readback buffers keyed by capacity
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
