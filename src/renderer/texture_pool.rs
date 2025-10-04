//! Texture pool for transient GPU textures (e.g., MSAA resolve attachments).
//!
//! Keyed by (width, height, depth_or_array_layers, format, sample_count, usage_bits).
//! Simple global LRU by entry count: acquire() searches for a matching entry and returns it,
//! otherwise creates a new texture. release() returns the texture to the pool and evicts LRU when
//! capacity is exceeded.
//!
//! Instrumentation: tracks hits, misses, evictions.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureKey {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub usage_bits: u32,
}

impl TextureKey {
    pub fn new(
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        TextureKey {
            width: size.width,
            height: size.height,
            depth_or_array_layers: size.depth_or_array_layers,
            format,
            sample_count,
            usage_bits: usage.bits(),
        }
    }

    pub fn usage(&self) -> wgpu::TextureUsages {
        // SAFETY: usage_bits was created from a valid TextureUsages
        wgpu::TextureUsages::from_bits_truncate(self.usage_bits)
    }

    pub fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: self.depth_or_array_layers,
        }
    }
}

#[derive(Debug)]
pub struct TexturePool {
    entry_limit: usize,
    entries: VecDeque<(TextureKey, wgpu::Texture)>, // front = LRU, back = MRU
    // instrumentation
    hits: u64,
    misses: u64,
    evictions: u64,
    allocations: u64,
    bytes_allocated: u64,
}

impl TexturePool {
    pub fn new(entry_limit: usize) -> Self {
        Self {
            entry_limit: entry_limit.max(1),
            entries: VecDeque::new(),
            hits: 0,
            misses: 0,
            evictions: 0,
            allocations: 0,
            bytes_allocated: 0,
        }
    }

    pub fn acquire(&mut self, device: &wgpu::Device, key: TextureKey) -> wgpu::Texture {
        // search best (any) matching entry (first fit)
        if let Some(idx) = self.entries.iter().position(|(k, _)| *k == key)
            && let Some((_k, tex)) = self.entries.remove(idx)
        {
            self.hits += 1;
            // move to MRU happens by pushing when we release; on acquire we just return it
            return tex;
        }
        self.misses += 1;
        // create new texture
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TexturePool entry"),
            size: key.size(),
            mip_level_count: 1,
            sample_count: key.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: key.format,
            view_formats: &[],
            usage: key.usage(),
        });
        self.allocations += 1;
        // Approximate bytes: width*height*4*mip(1)*depth, format-dependent; we skip exact calc for now
        let size = key.size();
        let bytes =
            (size.width as u64) * (size.height as u64) * 4 * (size.depth_or_array_layers as u64);
        self.bytes_allocated += bytes;
        tex
    }

    pub fn release(&mut self, key: TextureKey, tex: wgpu::Texture) {
        self.entries.push_back((key, tex));
        if self.entries.len() > self.entry_limit {
            let _ = self.entries.pop_front();
            self.evictions += 1;
        }
    }

    pub fn reset_metrics(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.allocations = 0;
        self.bytes_allocated = 0;
    }
}

impl crate::renderer::buffer_pool::BufferPool for TexturePool {
    fn stats(&self) -> crate::renderer::buffer_pool::PoolStats {
        crate::renderer::buffer_pool::PoolStats {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            allocations: self.allocations,
            bytes_allocated: self.bytes_allocated,
        }
    }

    fn reset_metrics(&mut self) {
        TexturePool::reset_metrics(self)
    }
}
