//! Texture pool for transient GPU textures (e.g., MSAA resolve attachments).
//!
//! Keyed by (width, height, depth_or_array_layers, format, sample_count, usage_bits).
//! Simple global LRU by entry count: acquire() searches for a matching entry and returns it,
//! otherwise creates a new texture. release() returns the texture to the pool and evicts LRU when
//! capacity is exceeded.

use std::collections::VecDeque;

/// The shape and use that make two pooled textures interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TextureKey {
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

/// Reuses transient textures, such as MSAA attachments.
#[derive(Debug)]
pub(crate) struct TexturePool {
    entry_limit: usize,
    entries: VecDeque<(TextureKey, wgpu::Texture)>, // front = LRU, back = MRU
}

impl TexturePool {
    pub fn new(entry_limit: usize) -> Self {
        Self {
            entry_limit: entry_limit.max(1),
            entries: VecDeque::new(),
        }
    }

    pub fn acquire(&mut self, device: &wgpu::Device, key: TextureKey) -> wgpu::Texture {
        // search best (any) matching entry (first fit)
        if let Some(idx) = self.entries.iter().position(|(k, _)| *k == key)
            && let Some((_k, tex)) = self.entries.remove(idx)
        {
            // move to MRU happens by pushing when we release; on acquire we just return it
            return tex;
        }
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TexturePool entry"),
            size: key.size(),
            mip_level_count: 1,
            sample_count: key.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: key.format,
            view_formats: &[],
            usage: key.usage(),
        })
    }

    pub fn release(&mut self, key: TextureKey, tex: wgpu::Texture) {
        self.entries.push_back((key, tex));
        if self.entries.len() > self.entry_limit {
            let _ = self.entries.pop_front();
        }
    }
}
