use super::platform::all as platform_all;
use super::readback_pool::ReadbackBufferPool;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to get a test device
    async fn device() -> wgpu::Device {
        let instance = platform_all::create_instance().await;
        let adapter = platform_all::request_adapter(&instance, None)
            .await
            .expect("adapter");
        let (device, _queue) = platform_all::request_device(&adapter)
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
