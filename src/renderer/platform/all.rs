use crate::renderer::error::InitializationError;
use std::sync::Arc;

pub async fn create_instance() -> wgpu::Instance {
    #[cfg(wasm)]
    use wgpu::util::new_instance_with_webgpu_detection;
    #[cfg(wasm)]
    let instance = new_instance_with_webgpu_detection(
        wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
    )
    .await;

    #[cfg(not(wasm))]
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

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
            apply_limit_buckets: false,
        })
        .await?;

    Ok(adapter)
}

pub async fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), InitializationError> {
    // Features: probe a curated set of stable, end-user-relevant features
    // and request only the ones the adapter advertises. Two constraints
    // shape the list:
    //   * `adapter.features()` includes wgpu-experimental features
    //     (`EXPERIMENTAL_RAY_QUERY`, `EXPERIMENTAL_MESH_SHADER`,
    //     `EXPERIMENTAL_COOPERATIVE_MATRIX`) that wgpu requires routed
    //     through `experimental_features` — passing them via
    //     `required_features` errors with `ExperimentalFeaturesNotEnabled`
    //     on macOS Metal. We don't use any experimental features today.
    //   * The Android emulator's SwiftShader Vulkan rejects `IMMEDIATES`
    //     at device creation; the existing uniform-buffer fallback path
    //     covers callers when the feature isn't available.
    // Limits: take the adapter's advertised limits directly. wgpu's
    // `Limits::default()` baseline exceeds what some emulators can
    // provide (`max_buffer_size`, `max_storage_buffer_binding_size`),
    // and Vulkan reports those mismatches via the same
    // `VK_ERROR_FEATURE_NOT_PRESENT` we hit before — but with an empty
    // wgpu-hal "Missing features:" log line because the constraint is on
    // limits, not features.
    let candidates = wgpu::Features::IMMEDIATES
        | wgpu::Features::TEXTURE_COMPRESSION_BC
        | wgpu::Features::TEXTURE_COMPRESSION_BC_SLICED_3D
        | wgpu::Features::TEXTURE_COMPRESSION_ETC2
        | wgpu::Features::TEXTURE_COMPRESSION_ASTC
        | wgpu::Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D
        | wgpu::Features::TEXTURE_COMPRESSION_ASTC_HDR
        | wgpu::Features::TEXTURE_FORMAT_16BIT_NORM
        | wgpu::Features::FLOAT32_FILTERABLE;
    let requested_features = adapter.features() & candidates;
    let requested_limits = adapter.limits();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("WGPU Device"),
            memory_hints: memory_hints(),
            required_features: requested_features,
            required_limits: requested_limits,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        })
        .await?;

    device.on_uncaptured_error(Arc::new(|error| {
        // Build metadata (compile-time)
        let pkg = env!("CARGO_PKG_NAME");
        let ver = env!("CARGO_PKG_VERSION");
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
        let git = option_env!("FC_GIT_HASH").unwrap_or("unknown");
        let built = option_env!("FC_BUILD_TIME").unwrap_or("unknown");
        // Runtime context set by runners like healthchecks
        let runner = std::env::var("FC_RUNNER").unwrap_or_else(|_| "".into());
        let current = std::env::var("FC_CURRENT_TEST").unwrap_or_else(|_| "".into());

        println!(
            "\n\n==== GPU error ({} v{} | {} | git {} | built {}) ====\nRunner: {}\nContext: {}\n\n{:#?}\n",
            pkg, ver, profile, git, built, runner, current, error
        );
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
    let format = choose_surface_format(&capabilities);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width: u32::max(size.width, 1),
        height: u32::max(size.height, 1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: capabilities.alpha_modes[0],
        desired_maximum_frame_latency: 2,
        view_formats: vec![format],
    };
    surface.configure(device, &config);

    config
}

// Extracted so it can be tested without constructing a full SurfaceCapabilities
fn choose_surface_format_from(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
    // Try preferred linear formats first.
    if let Some(&fmt) = formats.iter().find(|&&f| {
        matches!(
            f,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
        )
    }) {
        return fmt;
    }

    // If no linear 8-bit format is available, accept sRGB variants if present.
    if let Some(&fmt) = formats.iter().find(|&&f| {
        matches!(
            f,
            wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
        )
    }) {
        return fmt;
    }

    // Otherwise, fall back to the first advertised format.
    formats
        .first()
        .copied()
        .unwrap_or(wgpu::TextureFormat::Rgba8Unorm)
}

// The previous `features()` and `format_features()` helpers folded into
// the inline `candidates` mask in `request_device` above. Limits selection
// (also there) takes `adapter.limits()` directly — no more
// `Limits::default().using_resolution(...)`, which exceeded what
// downlevel emulators could provide.

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}

/// Prefer a linear (non-sRGB) 8-bit RGBA/BGRA format if available; otherwise, fall back to the
/// first supported format. We must choose a format that is actually advertised by the surface.
fn choose_surface_format(capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    choose_surface_format_from(&capabilities.formats)
}

#[cfg(test)]
mod platform_tests;

/// Negotiate a supported MSAA sample count for a given format on this adapter.
/// Halves `wanted` until `features.sample_count_supported(n)` or returns 1.
pub fn pick_sample_count(
    adapter: &wgpu::Adapter,
    mut wanted: u32,
    fmt: wgpu::TextureFormat,
) -> u32 {
    let flags = adapter.get_texture_format_features(fmt).flags;

    if wanted == 0 {
        wanted = 1;
    }
    if wanted > 16 {
        wanted = 16;
    }
    // Round down to nearest power of two
    while wanted > 1 && !wanted.is_power_of_two() {
        wanted /= 2;
    }

    let mut n = wanted;
    while n > 1 && !flags.sample_count_supported(n) {
        n /= 2;
    }
    n.max(1)
}
