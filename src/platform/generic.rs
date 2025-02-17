// This file contains a generic implementation for platforms that are not the web or mobile.
// Since we do not have any specific ties with the windowing system, we cannot implement
// stages that actually draw on screen, so we just provide headless context and stage.

use crate::{platform, Renderer};

impl Renderer {
    pub async fn headless() -> Renderer {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = platform::all::request_device(&adapter)
            .await
            .expect("Failed to request device");

        Renderer::new(device, queue)
    }
}
