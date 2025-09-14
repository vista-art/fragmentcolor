use crate::renderer::error::InitializationError;

pub async fn create_instance() -> wgpu::Instance {
    #[cfg(wasm)]
    use wgpu::util::new_instance_with_webgpu_detection;
    #[cfg(wasm)]
    let instance = new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor::default()).await;

    #[cfg(not(wasm))]
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    instance
}

pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, InitializationError> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await?;

    Ok(adapter)
}

pub async fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), InitializationError> {
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("WGPU Device"),
            memory_hints: memory_hints(),
            required_features: features(),
            required_limits: limits().using_resolution(adapter.limits()),
            trace: wgpu::Trace::Off,
        })
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
