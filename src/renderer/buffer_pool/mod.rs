//! Buffer pools used by the renderer.
//!
//! - UniformBufferPool: suballocates UNIFORM/COPY_DST buffers into fixed-size chunks, reset per frame.
//! - StorageBufferPool: the same strategy for STORAGE/COPY_DST buffers.
//! - ReadbackBufferPool: manages an LRU of whole MAP_READ buffers sized for readbacks.
//!
//! The strategies differ materially:
//! - UniformBufferPool batches many small writes each frame with alignment (often 256), never mapped for read.
//! - ReadbackBufferPool hands out whole buffers (Arc) suitable for COPY_DST + MAP_READ and async mapping.

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
