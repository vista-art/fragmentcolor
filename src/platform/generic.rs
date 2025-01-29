//
// Copyright (C) Artizans - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited.
//

// This file contains a generic implementation for platforms that are not the web or mobile.
// Since we do not have any specific ties with the windowing system, we cannot implement
// stages that actually draw on screen, so we just provide headless context and stage.

use crate::{ffi, Context, Stage};

impl Context {
    pub async fn headless() -> Context {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = ffi::platform::all::request_device(&adapter).await;

        Context::new(device, queue)
    }
}

impl Stage {
    pub async fn headless() -> Stage {
        let context = Context::headless().await;
        Stage::new(context)
    }
}
