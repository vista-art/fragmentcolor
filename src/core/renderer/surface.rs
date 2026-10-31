//! `HasDisplaySize`, the surface a window target draws into,
//! and the offscreen target a window without a surface falls back to.

use crate::core::renderer::{InitializationError, RenderContext, Renderer, WgpuFailure, device};
use crate::texture::check_plane;
use crate::{RenderTarget, Size, TextureTarget, WindowTarget};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

/// A window that reports its display size.
/// `Renderer::create_target` takes one that also has the `raw-window-handle`
/// traits `HasWindowHandle` and `HasDisplayHandle`, as a `winit` window does.
pub trait HasDisplaySize {
    fn size(&self) -> Size;
}

impl Renderer {
    /// A window target for `window`, or a texture target of its size when no surface
    /// opens from its handle, so CI, tests and headless runs still render.
    pub(super) async fn open_target(
        &self,
        window: impl HasWindowHandle + HasDisplayHandle + HasDisplaySize + Send + Sync + 'static,
    ) -> Result<RenderTarget, InitializationError> {
        let window_size = window.size();
        let size = wgpu::Extent3d {
            width: window_size.width,
            height: window_size.height,
            depth_or_array_layers: 1,
        };

        match self.create_surface(window, size).await {
            Ok((context, surface, config)) => Ok(RenderTarget::from(WindowTarget::new(
                context, surface, config,
            ))),
            Err(InitializationError::SurfaceError(error)) => {
                log::warn!(
                    "create_target: surface creation failed ({}). Falling back to TextureTarget.",
                    error
                );
                // A window with no area yet gets one pixel, as a surface does.
                let target = self
                    .create_texture_target([size.width.max(1), size.height.max(1)])
                    .await?;
                Ok(RenderTarget::from(target))
            }
            Err(error) => Err(error),
        }
    }

    /// An offscreen target of `size`, checked against the device's largest plane.
    pub(super) async fn allocate_target(
        &self,
        size: Size,
    ) -> Result<TextureTarget, InitializationError> {
        let context = self.context(None).await?;
        let limit = context.device.limits().max_texture_dimension_2d;
        check_plane(size, limit)?;
        // For offscreen texture targets on Web, force sample_count = 1 to avoid MSAA resolve issues.
        // Canvas targets may still use MSAA via the surface path.
        context.set_sample_count(1);
        let target = TextureTarget::new(context, size, wgpu::TextureFormat::Rgba8Unorm);

        Ok(target)
    }

    pub(crate) async fn create_surface<'window>(
        &self,
        handle: impl Into<wgpu::SurfaceTarget<'window>>,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'window>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let instance = self.instance().await;
        let surface = open_surface(&instance, handle)?;
        self.configure_surface(surface, size).await
    }

    /// Like `create_surface`, but takes an already-constructed `wgpu::Surface`
    /// (usually one built from a raw pointer via `instance.create_surface_unsafe`).
    /// Callers are responsible for ensuring the backing window/layer outlives the surface.
    #[cfg(ios)]
    pub(crate) async fn configure_unsafe_surface(
        &self,
        target: wgpu::SurfaceTargetUnsafe,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let instance = self.instance().await;
        // SAFETY: the caller promises the underlying handle (CAMetalLayer / ANativeWindow / ...)
        // remains valid for the lifetime of the returned Surface.
        let surface = unsafe { open_unsafe_surface(&instance, target) }?;
        self.configure_surface(surface, size).await
    }

    async fn configure_surface<'window>(
        &self,
        surface: wgpu::Surface<'window>,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'window>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let context = self.context(Some(&surface)).await?;

        let adapter = self.adapter.read();
        let adapter_ref = adapter.as_ref().ok_or(InitializationError::AdapterNotSet)?;
        let config = configure_surface(&context.device, adapter_ref, &surface, &size)?;

        // Negotiate and store effective sample count (currently default wanted=1; configurable later)
        if let Some(adapter_ref) = adapter.as_ref() {
            let sample_count = device::pick_sample_count(adapter_ref, 1, config.format);
            context.set_sample_count(sample_count);
        }

        Ok((context, surface, config))
    }
}

/// A surface for `handle` on `instance`.
fn open_surface<'window>(
    instance: &wgpu::Instance,
    handle: impl Into<wgpu::SurfaceTarget<'window>>,
) -> Result<wgpu::Surface<'window>, WgpuFailure> {
    Ok(instance.create_surface(handle)?)
}

/// A surface for a raw `target` on `instance`.
///
/// # Safety
/// The window or layer behind `target` must outlive the surface.
#[cfg(ios)]
unsafe fn open_unsafe_surface(
    instance: &wgpu::Instance,
    target: wgpu::SurfaceTargetUnsafe,
) -> Result<wgpu::Surface<'static>, WgpuFailure> {
    // SAFETY: the caller keeps the window or layer behind `target` alive as long as the surface.
    Ok(unsafe { instance.create_surface_unsafe(target) }?)
}

/// Configures `surface` at `size` with a format the adapter supports.
fn configure_surface(
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
mod tests {
    use super::{InitializationError, choose_surface_format, choose_surface_format_from};

    #[test]
    fn chooses_surface_format_with_linear_preference() {
        // Arrange: linear exists alongside sRGB
        let formats = vec![
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
        ];
        // Act / Assert: prefer linear BGRA over sRGB RGBA
        let f = choose_surface_format_from(&formats);
        assert_eq!(f, Some(wgpu::TextureFormat::Bgra8Unorm));

        // Arrange: no linear, only sRGB
        let formats2 = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
        let f2 = choose_surface_format_from(&formats2);
        assert_eq!(f2, Some(wgpu::TextureFormat::Rgba8UnormSrgb));

        // Arrange: an empty list has no format to choose
        let empty: Vec<wgpu::TextureFormat> = vec![];
        let f3 = choose_surface_format_from(&empty);
        assert_eq!(f3, None);
    }

    // Story: wgpu reports default capabilities for a surface the adapter cannot present to,
    // such as a second WebGL2 canvas. Configuring it anyway only logs UnsupportedQueueFamily.
    #[test]
    fn rejects_a_surface_without_formats() {
        // Arrange
        let capabilities = wgpu::SurfaceCapabilities::default();

        // Act
        let result = choose_surface_format(&capabilities);

        // Assert
        assert!(
            matches!(&result, Err(InitializationError::Error(message)) if message.contains("lists no formats")),
            "got {result:?}"
        );
    }
}
