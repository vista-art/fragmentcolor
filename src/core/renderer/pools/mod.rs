//! The GPU buffers and textures a renderer reuses from frame to frame.

use std::borrow::Cow;

pub(crate) mod uniform;
pub(crate) use uniform::{BufferLocation, UniformBufferPool};

pub(crate) mod readback;
#[cfg(wasm)]
pub(crate) use readback::MapReceiver;
#[cfg(not(wasm))]
pub(crate) use readback::{MAP_LIMIT, MapError, poll_map};
pub(crate) use readback::{PendingReadback, ReadbackBufferPool};

pub(crate) mod storage;
pub(crate) use storage::StorageBufferPool;

mod texture;
pub(crate) use texture::*;

/// `size` rounded up to whole 4-byte words.
/// A root of `f16` values can span 2 or 6 bytes,
/// and wgpu copies, writes, maps and binds storage in words.
pub(crate) fn align_size(size: u64) -> u64 {
    wgpu::util::align_to(size, wgpu::COPY_BUFFER_ALIGNMENT)
}

/// `bytes` with zeros after them up to the size `align_size` gives.
pub(crate) fn pad_bytes(bytes: &[u8]) -> Cow<'_, [u8]> {
    let size = align_size(bytes.len() as u64) as usize;
    if size == bytes.len() {
        return Cow::Borrowed(bytes);
    }
    let mut padded = bytes.to_vec();
    padded.resize(size, 0);
    Cow::Owned(padded)
}
