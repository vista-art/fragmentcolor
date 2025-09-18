//! Buffer pools used by the renderer.
//!
//! This module provides a common trait (BufferPool) and concrete pool strategies:
//! - UniformBufferPool: suballocates UNIFORM/COPY_DST buffers into fixed-size chunks, reset per frame.
//! - ReadbackBufferPool: manages an LRU of whole MAP_READ buffers sized for readbacks.
//!
//! The strategies differ materially:
//! - UniformBufferPool batches many small writes each frame with alignment (often 256), never mapped for read.
//! - ReadbackBufferPool hands out whole buffers (Arc) suitable for COPY_DST + MAP_READ and async mapping.
//!
//! Both implement a minimal BufferPool trait (reset). Additional diagnostics can be layered uniformly.

#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub allocations: u64,
    pub bytes_allocated: u64,
}

pub trait BufferPool {
    /// Reset pool state for the next frame (no-op for pools that do not track per-frame cursors).
    fn reset(&mut self) {}
    /// Returns instrumentation counters for this pool.
    fn stats(&self) -> PoolStats {
        PoolStats::default()
    }
    /// Resets instrumentation counters.
    fn reset_metrics(&mut self) {}
}

pub(crate) mod uniform;
pub(crate) use uniform::{BufferLocation, UniformBufferPool};

pub(crate) mod readback;
pub(crate) use readback::ReadbackBufferPool;
