//! What a render draws into, and the `Target` traits each destination implements.

mod texture_target;
pub use texture_target::*;

mod window_target;
pub use window_target::*;

mod headless_window;
pub use headless_window::*;

mod render_target;
pub use render_target::*;

mod platform;
#[cfg(any(python, wasm, mobile))]
pub use platform::*;

use crate::RenderContext;
use crate::Size;
use crate::texture::TextureError;
use lsp_doc::lsp_doc;
use std::sync::Arc;

mod error;
pub use error::*;

/// Mirrors the variants of the old `wgpu::SurfaceError` (removed in wgpu 29, which folded
/// failure states into the `CurrentSurfaceTexture` enum). Kept as a crate-local error type so
/// the `Target` trait and dependent error enums stay stable across wgpu upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError {
    /// A timeout was encountered while trying to acquire the next frame.
    Timeout,
    /// The underlying surface has changed and the swapchain is outdated.
    Outdated,
    /// The surface has been lost and needs to be recreated.
    Lost,
    /// The window is occluded; the caller should skip the frame.
    Occluded,
    /// A validation error was raised by the wgpu runtime.
    Validation,
    /// Device / allocator reports out-of-memory.
    OutOfMemory,
    /// Any other driver-reported failure.
    Other,
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SurfaceError::Timeout => "timeout acquiring surface frame",
            SurfaceError::Outdated => "surface is outdated",
            SurfaceError::Lost => "surface is lost",
            SurfaceError::Occluded => "surface is occluded",
            SurfaceError::Validation => "surface validation error",
            SurfaceError::OutOfMemory => "out of memory",
            SurfaceError::Other => "unspecified surface error",
        };
        f.write_str(s)
    }
}

impl std::error::Error for SurfaceError {}

/// Convert a [`wgpu::CurrentSurfaceTexture`] into the previous `Result<SurfaceTexture, _>` shape.
/// `Suboptimal` frames are treated as success (the texture is still usable, the caller may
/// choose to reconfigure out-of-band).
pub(crate) fn surface_texture_from(
    result: wgpu::CurrentSurfaceTexture,
) -> Result<wgpu::SurfaceTexture, SurfaceError> {
    match result {
        wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
            Ok(t)
        }
        wgpu::CurrentSurfaceTexture::Timeout => Err(SurfaceError::Timeout),
        wgpu::CurrentSurfaceTexture::Outdated => Err(SurfaceError::Outdated),
        wgpu::CurrentSurfaceTexture::Lost => Err(SurfaceError::Lost),
        wgpu::CurrentSurfaceTexture::Occluded => Err(SurfaceError::Occluded),
        wgpu::CurrentSurfaceTexture::Validation => Err(SurfaceError::Validation),
    }
}

/// Frame-acquisition entry point. Sealed (crate-private supertrait of
/// [`Target`]) so external callers can't implement [`Target`] for their own
/// types — every wgpu-touching method lives here. Custom adapters go
/// through the existing [`crate::Renderer::create_target`] /
/// [`crate::Renderer::create_texture_target`] constructors, which produce
/// the canonical impls.
pub(crate) trait FrameSource {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError>;

    /// The context that created the target, or `None` before it has one.
    /// A Renderer draws only into its own targets, since a frame works on one device.
    fn owner(&self) -> Option<Arc<RenderContext>>;
}

#[lsp_doc("docs/api/targets/target/target.md")]
#[allow(private_bounds)]
pub trait Target: FrameSource {
    fn size(&self) -> Size;

    fn resize(&mut self, size: impl Into<Size>) -> Result<(), TextureError>;

    /// Read back the target's current contents as packed RGBA8 bytes
    /// (row-major, top-left origin). Async because GPU readback is
    /// asynchronous on every backend (browser `mapAsync` / native
    /// `device.poll(Wait)`).
    ///
    /// A `TextureTarget` returns its pixels, or `TextureError::Readback`
    /// when the readback fails. A `WindowTarget` always returns that error,
    /// since a frame on screen can't be read back.
    #[allow(async_fn_in_trait)]
    async fn get_image(&self) -> Result<Vec<u8>, TextureError>;
}

pub(crate) trait TargetFrame {
    fn view(&self) -> &wgpu::TextureView;
    fn format(&self) -> wgpu::TextureFormat;
    /// The size of the texture this frame draws into, which can differ from the
    /// target's `size()` after a resize the target hasn't applied yet.
    fn size(&self) -> wgpu::Extent3d;
    fn present(self: Box<Self>, queue: &wgpu::Queue);
    fn auto_present(&self) -> bool {
        true
    }
}
