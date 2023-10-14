use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub trait HasWindow: HasRawDisplayHandle + HasRawWindowHandle {
    fn size(&self) -> mint::Vector2<u32>;
}
// @TODO consider renaming it to Window or Framebuffer
// and the internal `instance` property to `surface`
//
// In Ruffle, this is called SwapChainTarget
// which is a concrete impl of the RenderTarget trait
//
// Ruffle also contains the TextureTarget struct
// which implements the same trait.
pub struct SurfaceContext {
    pub(super) instance: wgpu::Surface,
    pub(super) config: wgpu::SurfaceConfiguration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetRef(pub u8);

pub struct Target {
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
}

impl Target {
    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}

/// Parameters of a texture target that affect its pipeline compatibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetInfo {
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub aspect_ratio: f32,
}
