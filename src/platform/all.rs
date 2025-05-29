use crate::InitializationError;

pub async fn request_headless_adapter(
    instance: &wgpu::Instance,
) -> Result<wgpu::Adapter, InitializationError> {
    request_adapter(instance, None).await
}

pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, InitializationError> {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await
        .ok_or(InitializationError::AdapterError())
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

pub fn configure_surface(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    surface: &wgpu::Surface,
    size: &wgpu::Extent3d,
) -> wgpu::SurfaceConfiguration {
    let capabilities = surface.get_capabilities(adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: capabilities.formats[0].remove_srgb_suffix(),
        width: u32::max(size.width, 1),
        height: u32::max(size.height, 1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: capabilities.alpha_modes[0],
        desired_maximum_frame_latency: 2,
        view_formats: vec![],
    };
    surface.configure(device, &config);

    config
}

pub fn request_headless_adapter_sync(
    instance: &wgpu::Instance,
) -> Result<wgpu::Adapter, InitializationError> {
    request_adapter_sync(instance, None)
}

pub fn request_adapter_sync(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, InitializationError> {
    pollster::block_on(request_adapter(instance, surface))
}

pub fn request_device_sync(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), InitializationError> {
    pollster::block_on(request_device(adapter))
}

fn limits() -> wgpu::Limits {
    #[cfg(wasm)]
    let limits = wgpu::Limits::downlevel_webgl2_defaults();

    #[cfg(not(wasm))]
    let limits = wgpu::Limits::default();

    limits
}

fn features() -> wgpu::Features {
    wgpu::Features::empty()
}

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}
