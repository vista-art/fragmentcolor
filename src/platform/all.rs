use crate::InitializationError;

fn limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}

fn features() -> wgpu::Features {
    wgpu::Features::empty()
}

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}

pub async fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), InitializationError> {
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WGPU Device"),
                memory_hints: memory_hints(),
                required_features: features(),
                required_limits: limits().using_resolution(adapter.limits()),
            },
            None,
        )
        .await?;

    device.on_uncaptured_error(Box::new(|error| {
        println!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
    }));

    Ok((device, queue))
}
