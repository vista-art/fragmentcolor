//! `TexturePool`: transient textures, such as MSAA
//! and depth attachments, reused across frames and freed when idle.

use parking_lot::RwLock;
use std::collections::VecDeque;

/// Renders an entry may sit unused before the pool frees it.
const IDLE_RENDERS: u64 = 8;

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

#[derive(Debug)]
struct Entry {
    key: TextureKey,
    texture: wgpu::Texture,
    /// The render that last released it.
    used: u64,
}

/// Reuses transient textures, such as MSAA attachments.
#[derive(Debug)]
pub(crate) struct TexturePool {
    entry_limit: usize,
    render: u64,
    entries: VecDeque<Entry>, // front = LRU, back = MRU
}

impl TexturePool {
    pub fn new(entry_limit: usize) -> Self {
        Self {
            entry_limit: entry_limit.max(1),
            render: 0,
            entries: VecDeque::new(),
        }
    }

    /// Starts a render, and frees the entries no recent render used.
    pub fn start_render(&mut self) {
        self.render += 1;
        let render = self.render;
        self.entries
            .retain(|entry| render - entry.used <= IDLE_RENDERS);
    }

    /// The number of renders started so far, which names the current one.
    pub fn render(&self) -> u64 {
        self.render
    }

    pub fn acquire(&mut self, device: &wgpu::Device, key: TextureKey) -> wgpu::Texture {
        // search best (any) matching entry (first fit)
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key)
            && let Some(entry) = self.entries.remove(index)
        {
            // move to MRU happens by pushing when we release; on acquire we just return it
            return entry.texture;
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

    pub fn release(&mut self, key: TextureKey, texture: wgpu::Texture) {
        self.entries.push_back(Entry {
            key,
            texture,
            used: self.render,
        });
        if self.entries.len() > self.entry_limit {
            let _ = self.entries.pop_front();
        }
    }
}

/// A pooled texture and its view, back in the pool when dropped.
pub(crate) struct Lease<'pool> {
    pool: &'pool RwLock<TexturePool>,
    key: TextureKey,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl<'pool> Lease<'pool> {
    /// Takes a texture shaped like `key` from `pool`, or creates one.
    pub(crate) fn take(
        pool: &'pool RwLock<TexturePool>,
        device: &wgpu::Device,
        key: TextureKey,
    ) -> Self {
        let texture = pool.write().acquire(device, key);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            pool,
            key,
            texture,
            view,
        }
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        self.pool.write().release(self.key, self.texture.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl TexturePool {
        pub(crate) fn keys(&self) -> impl Iterator<Item = &TextureKey> {
            self.entries.iter().map(|entry| &entry.key)
        }
    }
}
