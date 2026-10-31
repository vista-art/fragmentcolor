//! `ReadbackBufferPool`: whole mappable buffers for readbacks, reused by size.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::RwLock;

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

        if let Some(idx) = best_index
            // Move to MRU position and return
            && let Some(entry) = self.entries.remove(idx)
        {
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

    /// Remove `buffer` from the pool, so no later readback receives it.
    pub fn discard(&mut self, buffer: &Arc<wgpu::Buffer>) {
        self.entries
            .retain(|entry| !Arc::ptr_eq(&entry.buffer, buffer));
    }
}

/// A pooled buffer with a read mapping in flight.
///
/// Dropping it before [`PendingReadback::finish`] unmaps the buffer and removes it from the pool.
/// A readback that timed out or failed leaves its mapping pending or mapped, and a later
/// readback copying into that buffer would fail validation with `BufferStillMapped`.
pub(crate) struct PendingReadback<'pool> {
    pool: &'pool RwLock<ReadbackBufferPool>,
    buffer: Arc<wgpu::Buffer>,
    finished: bool,
}

impl<'pool> PendingReadback<'pool> {
    pub fn new(pool: &'pool RwLock<ReadbackBufferPool>, buffer: Arc<wgpu::Buffer>) -> Self {
        Self {
            pool,
            buffer,
            finished: false,
        }
    }

    /// Keep the buffer pooled. Call it once the bytes are read and the buffer is unmapped.
    pub fn finish(mut self) {
        self.finished = true;
    }
}

impl Drop for PendingReadback<'_> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.buffer.unmap();
        self.pool.write().discard(&self.buffer);
    }
}

/// What a buffer's `map_async` callback sends.
pub(crate) type MapReceiver =
    futures::channel::oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>;

/// How long a native readback waits for its buffer to map.
#[cfg(not(wasm))]
pub(crate) const MAP_LIMIT: std::time::Duration = std::time::Duration::from_secs(10);

/// The longest one device poll inside a map wait blocks.
#[cfg(not(wasm))]
const POLL_STEP: std::time::Duration = std::time::Duration::from_millis(100);

/// Why a readback stopped waiting for its buffer to map.
#[cfg(not(wasm))]
#[derive(Debug, thiserror::Error)]
pub(crate) enum MapError {
    #[error("the buffer did not map within {0:?}")]
    Timeout(std::time::Duration),
    #[error("map_async failed: {0}")]
    Failed(#[from] wgpu::BufferAsyncError),
    #[error("the map callback was dropped")]
    Dropped,
    #[error("device poll failed: {0}")]
    Poll(#[from] wgpu::PollError),
}

/// Polls `device` until `receiver` holds the map result, for at most `limit`.
///
/// The map callback fires only from a device poll, and nothing else may poll this device.
/// So every turn of the wait polls it, for at most `POLL_STEP`, and a map
/// that never lands returns `MapError::Timeout` instead of blocking forever.
#[cfg(not(wasm))]
pub(crate) fn wait_for_map(
    device: &wgpu::Device,
    receiver: &mut MapReceiver,
    limit: std::time::Duration,
) -> Result<(), MapError> {
    let deadline = std::time::Instant::now() + limit;
    loop {
        match receiver.try_recv() {
            Ok(Some(result)) => return Ok(result?),
            Ok(None) => {}
            Err(_) => return Err(MapError::Dropped),
        }
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            return Err(MapError::Timeout(limit));
        }
        let wait = wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(left.min(POLL_STEP)),
        };
        match device.poll(wait) {
            // Nothing is in flight, so the map waits on no submission: check again shortly.
            Ok(wgpu::PollStatus::QueueEmpty) => {
                std::thread::sleep(left.min(std::time::Duration::from_millis(1)))
            }
            Ok(_) | Err(wgpu::PollError::Timeout) => {}
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PendingReadback;
    use crate::ReadbackBufferPool;
    use parking_lot::RwLock;
    use std::sync::Arc;

    // Helper to get a test device
    async fn device() -> wgpu::Device {
        let instance = crate::core::renderer::device::create_instance().await;
        let adapter = crate::core::renderer::device::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, _queue) = crate::core::renderer::device::request_device(&adapter)
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

    // Story: a readback that stops before its bytes are read never hands its buffer out again.
    #[test]
    fn unfinished_readback_leaves_the_pool_unmapped() {
        pollster::block_on(async move {
            let device = device().await;
            let pool = RwLock::new(ReadbackBufferPool::new("Test Readback Pool", 8));

            let buffer = pool.write().get(&device, 1024);
            buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            drop(PendingReadback::new(&pool, buffer.clone()));

            let fresh = pool.write().get(&device, 1024);
            assert!(!Arc::ptr_eq(&buffer, &fresh));

            // The pending map was cancelled, so the buffer maps again.
            let (sender, receiver) = std::sync::mpsc::channel();
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    let _ = sender.send(result);
                });
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .expect("poll");
            assert!(receiver.recv().expect("map callback").is_ok());
        });
    }

    /// Runs `wait` on a thread of its own and returns what it gives back,
    /// or `None` when it is still waiting after `watchdog`.
    fn run_within<T: Send + 'static>(
        watchdog: std::time::Duration,
        wait: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (done, finished) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = done.send(wait());
        });
        finished.recv_timeout(watchdog).ok()
    }

    // Story: a readback's map lands though no other code polls the device,
    // because the wait polls it itself.
    #[test]
    fn map_wait_polls_the_device_itself() {
        // Arrange
        let waiting = move || {
            pollster::block_on(async move {
                let device = device().await;
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("map wait"),
                    size: 256,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                let (sender, mut receiver) = futures::channel::oneshot::channel();
                buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = sender.send(result);
                    });

                // Act
                super::wait_for_map(&device, &mut receiver, super::MAP_LIMIT)
                    .map_err(|error| error.to_string())
            })
        };

        // Assert
        let result = run_within(std::time::Duration::from_secs(30), waiting);
        assert!(matches!(result, Some(Ok(()))), "got {result:?}");
    }

    // Story: a map that never lands ends the wait at its limit with a timeout,
    // so a readback returns an error instead of blocking forever.
    #[test]
    fn map_wait_stops_at_its_limit() {
        // Arrange
        let limit = std::time::Duration::from_millis(200);
        let waiting = move || {
            pollster::block_on(async move {
                let device = device().await;
                let (_sender, mut receiver) = futures::channel::oneshot::channel();

                // Act
                super::wait_for_map(&device, &mut receiver, limit)
            })
        };

        // Assert
        let result = run_within(std::time::Duration::from_secs(30), waiting);
        assert!(
            matches!(result, Some(Err(super::MapError::Timeout(waited))) if waited == limit),
            "got {result:?}"
        );
    }

    // Story: a finished readback keeps its buffer for the next one.
    #[test]
    fn finished_readback_stays_in_the_pool() {
        pollster::block_on(async move {
            let device = device().await;
            let pool = RwLock::new(ReadbackBufferPool::new("Test Readback Pool", 8));

            let buffer = pool.write().get(&device, 1024);
            PendingReadback::new(&pool, buffer.clone()).finish();

            let again = pool.write().get(&device, 1024);
            assert!(Arc::ptr_eq(&buffer, &again));
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
