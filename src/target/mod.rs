mod texture;
pub use texture::*;

mod window;
pub use window::*;

mod headless;
pub use headless::*;

mod all;
pub use all::*;

mod platform;
#[cfg(any(python, wasm))]
pub use platform::*;

use crate::size::Size;
use lsp_doc::lsp_doc;

pub mod error;

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
pub fn surface_texture_from(
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

#[lsp_doc("docs/api/targets/target/target.md")]
pub trait Target {
    fn size(&self) -> Size;

    fn resize(&mut self, size: impl Into<Size>);

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError>;

    fn get_image(&self) -> Vec<u8>;
}

pub trait TargetFrame {
    fn view(&self) -> &wgpu::TextureView;
    fn format(&self) -> wgpu::TextureFormat;
    fn present(self: Box<Self>);
    fn auto_present(&self) -> bool {
        true
    }
}
