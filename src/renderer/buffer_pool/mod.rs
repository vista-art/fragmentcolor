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

pub(crate) mod storage;
pub(crate) use storage::StorageBufferPool;

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Default trait methods on BufferPool provide zeroed stats and no-op reset.
    struct Dummy;
    impl BufferPool for Dummy {}

    #[test]
    fn buffer_pool_default_methods() {
        let mut d = Dummy;
        // default reset: no-op
        d.reset();
        // default stats: zeros
        let s = d.stats();
        assert_eq!(s.hits, 0);
        assert_eq!(s.misses, 0);
        assert_eq!(s.evictions, 0);
        assert_eq!(s.allocations, 0);
        assert_eq!(s.bytes_allocated, 0);
        // default reset_metrics: no-op
        d.reset_metrics();
    }
}
