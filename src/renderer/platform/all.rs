use crate::renderer::error::InitializationError;
use crate::shader::error::describe;
use std::sync::Arc;
#[cfg(not(wasm))]
use std::{fs::OpenOptions, io::Write, path::Path};

/// Creates the wgpu instance, detecting WebGPU on the web.
pub(crate) async fn create_instance() -> wgpu::Instance {
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

/// Requests an adapter that favors performance and can present to `surface`, if given.
pub(crate) async fn request_adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, InitializationError> {
    let options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: surface,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    };
    match instance.request_adapter(&options).await {
        Ok(adapter) => Ok(adapter),
        Err(error) => Err(InitializationError::AdapterError(describe(&error))),
    }
}

/// The features a device may open with, when the adapter offers them.
/// `Shader::new` refuses a shader that needs any other feature.
pub(crate) const CANDIDATES: wgpu::Features = wgpu::Features::IMMEDIATES
    .union(wgpu::Features::TEXTURE_COMPRESSION_BC)
    .union(wgpu::Features::TEXTURE_COMPRESSION_BC_SLICED_3D)
    .union(wgpu::Features::TEXTURE_COMPRESSION_ETC2)
    .union(wgpu::Features::TEXTURE_COMPRESSION_ASTC)
    .union(wgpu::Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D)
    .union(wgpu::Features::TEXTURE_COMPRESSION_ASTC_HDR)
    .union(wgpu::Features::TEXTURE_FORMAT_16BIT_NORM)
    .union(wgpu::Features::FLOAT32_FILTERABLE)
    .union(wgpu::Features::FLOAT32_BLENDABLE);

/// Opens the device with the stable features the adapter offers.
pub(crate) async fn request_device(
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
    let requested_features = adapter.features() & CANDIDATES;
    let requested_limits = cap_limits(adapter);

    let descriptor = wgpu::DeviceDescriptor {
        label: Some("WGPU Device"),
        memory_hints: memory_hints(),
        required_features: requested_features,
        required_limits: requested_limits,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    };
    let (device, queue) = match adapter.request_device(&descriptor).await {
        Ok(opened) => opened,
        Err(error) => return Err(InitializationError::DeviceError(describe(&error))),
    };

    device.on_uncaptured_error(Arc::new(report_error));

    Ok((device, queue))
}

/// A device that takes buffers of at most `limit` bytes, for tests of the buffer limit.
#[cfg(test)]
pub(crate) async fn request_small_device(limit: u64) -> (wgpu::Device, wgpu::Queue) {
    let instance = create_instance().await;
    let adapter = request_adapter(&instance, None).await.expect("adapter");
    let adapter_limits = adapter.limits();
    let limits = wgpu::Limits {
        max_buffer_size: limit,
        max_uniform_buffer_binding_size: adapter_limits.max_uniform_buffer_binding_size.min(limit),
        max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size.min(limit),
        ..adapter_limits
    };
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: limits,
            ..Default::default()
        })
        .await
        .expect("device")
}

/// The adapter's limits, with the vertex stride WebGL2 takes. wgpu reports no stride
/// limit there, and WebGL's `vertexAttribPointer` refuses a stride past 255 bytes.
fn cap_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    let limits = adapter.limits();
    #[cfg(wasm)]
    if adapter.get_info().backend == wgpu::Backend::Gl {
        return wgpu::Limits {
            max_vertex_buffer_array_stride: limits.max_vertex_buffer_array_stride.min(255),
            ..limits
        };
    }
    limits
}

/// The variable that names the file collecting uncaptured GPU errors.
#[cfg(not(wasm))]
const ERROR_LOG: &str = "FC_GPU_ERROR_LOG";

/// Reports a GPU error that no error scope caught.
/// The logger gets it with the build and runner that hit it,
/// and the file that `FC_GPU_ERROR_LOG` names gets it too, so the
/// iOS, Android and Python healthchecks fail the test that raised it.
fn report_error(error: wgpu::Error) {
    // Build metadata (compile-time)
    let pkg = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
    let git = option_env!("FC_GIT_HASH").unwrap_or("unknown");
    let built = option_env!("FC_BUILD_TIME").unwrap_or("unknown");
    // Runtime context set by runners like healthchecks
    let runner = std::env::var("FC_RUNNER").unwrap_or_else(|_| "".into());
    let current = std::env::var("FC_CURRENT_TEST").unwrap_or_else(|_| "".into());

    // log::error! reaches every platform's sink, including the browser
    // console on wasm where stdout is discarded.
    log::error!(
        "\n\n==== GPU error ({pkg} v{ver} | {profile} | git {git} | built {built}) ====\nRunner: {runner}\nContext: {current}\n\n{error:#?}\n"
    );

    #[cfg(not(wasm))]
    if let Some(path) = std::env::var_os(ERROR_LOG) {
        append_error(Path::new(&path), &error);
    }
}

/// Appends `error` to the file at `path`, creating it when missing.
#[cfg(not(wasm))]
fn append_error(path: &Path, error: &wgpu::Error) {
    let appended = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{error}"));
    if let Err(problem) = appended {
        log::warn!(
            "Failed to append a GPU error to {}: {problem}",
            path.display()
        );
    }
}

/// Configures `surface` at `size` with a format the adapter supports.
pub(crate) fn configure_surface(
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    surface: &wgpu::Surface,
    size: &wgpu::Extent3d,
) -> Result<wgpu::SurfaceConfiguration, InitializationError> {
    let capabilities = surface.get_capabilities(adapter);
    let format = choose_surface_format(&capabilities)?;
    let alpha_mode = capabilities
        .alpha_modes
        .first()
        .copied()
        .unwrap_or(wgpu::CompositeAlphaMode::Auto);
    // A window with no area yet gets one pixel, and one past the device limit is an error.
    let (width, height) = (size.width.max(1), size.height.max(1));
    crate::texture::check_size(width, height, device.limits().max_texture_dimension_2d)?;
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode,
        desired_maximum_frame_latency: 2,
        view_formats: vec![format],
    };
    surface.configure(device, &config);

    Ok(config)
}

// Extracted so it can be tested without constructing a full SurfaceCapabilities
fn choose_surface_format_from(formats: &[wgpu::TextureFormat]) -> Option<wgpu::TextureFormat> {
    // Try preferred linear formats first.
    if let Some(&fmt) = formats.iter().find(|&&f| {
        matches!(
            f,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
        )
    }) {
        return Some(fmt);
    }

    // If no linear 8-bit format is available, accept sRGB variants if present.
    if let Some(&fmt) = formats.iter().find(|&&f| {
        matches!(
            f,
            wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
        )
    }) {
        return Some(fmt);
    }

    // Otherwise, fall back to the first advertised format.
    formats.first().copied()
}

// The previous `features()` and `format_features()` helpers folded into
// the inline `candidates` mask in `request_device` above. Limits selection
// (also there) takes `adapter.limits()` directly — no more
// `Limits::default().using_resolution(...)`, which exceeded what
// downlevel emulators could provide.

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}

/// Why a surface that lists no format, such as a second WebGL2 canvas, cannot present.
const NO_SURFACE_FORMATS: &str =
    "the surface lists no formats for this adapter. On WebGL2, each canvas needs its own Renderer";

/// Prefer a linear (non-sRGB) 8-bit RGBA/BGRA format if available; otherwise, fall back to the
/// first supported format. We must choose a format that is actually advertised by the surface.
/// A surface that lists no format cannot present for this adapter, such as a second WebGL2 canvas.
fn choose_surface_format(
    capabilities: &wgpu::SurfaceCapabilities,
) -> Result<wgpu::TextureFormat, InitializationError> {
    choose_surface_format_from(&capabilities.formats)
        .ok_or_else(|| InitializationError::Error(NO_SURFACE_FORMATS.into()))
}

#[cfg(test)]
mod platform_tests;

/// Negotiate a supported MSAA sample count for a given format on this adapter.
/// Halves `wanted` until `features.sample_count_supported(n)` or returns 1.
pub(crate) fn pick_sample_count(
    adapter: &wgpu::Adapter,
    mut wanted: u32,
    fmt: wgpu::TextureFormat,
) -> u32 {
    let flags = adapter.get_texture_format_features(fmt).flags;

    wanted = wanted.clamp(1, 16);
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
