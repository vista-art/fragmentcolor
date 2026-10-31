//! The GPU buffers and textures a renderer reuses from frame to frame.

pub(crate) mod uniform;
pub(crate) use uniform::{BufferLocation, UniformBufferPool};

pub(crate) mod readback;
#[cfg(wasm)]
pub(crate) use readback::MapReceiver;
#[cfg(not(wasm))]
pub(crate) use readback::{MAP_LIMIT, wait_for_map};
pub(crate) use readback::{PendingReadback, ReadbackBufferPool};

pub(crate) mod storage;
pub(crate) use storage::StorageBufferPool;

mod texture;
pub(crate) use texture::*;
