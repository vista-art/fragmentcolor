fn limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}

fn features() -> wgpu::Features {
    wgpu::Features::empty()
}

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}

pub(crate) async fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                memory_hints: memory_hints(),
                required_features: features(),
                required_limits: limits().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device")
}
