//! Opens the GPU: the wgpu instance,
//! adapter and device, their limits and the sample count.

use crate::core::renderer::WgpuFailure;
use crate::core::renderer::device_errors::uncaptured::report_error;
use std::sync::Arc;

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
) -> Result<wgpu::Adapter, WgpuFailure> {
    let options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: surface,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    };
    Ok(instance.request_adapter(&options).await?)
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
) -> Result<(wgpu::Device, wgpu::Queue), WgpuFailure> {
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
    let (device, queue) = adapter.request_device(&descriptor).await?;

    device.on_uncaptured_error(Arc::new(report_error));

    Ok((device, queue))
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

fn memory_hints() -> wgpu::MemoryHints {
    wgpu::MemoryHints::Performance
}

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
