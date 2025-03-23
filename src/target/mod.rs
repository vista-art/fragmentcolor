use std::fmt::{Debug, Formatter};

pub trait Target {
    fn size(&self) -> wgpu::Extent3d;
    fn resize(&mut self, size: wgpu::Extent3d);
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError>;
}

pub trait TargetFrame {
    fn view(&self) -> &wgpu::TextureView;
    fn format(&self) -> wgpu::TextureFormat;
    fn present(self: Box<Self>);
    fn auto_present(&self) -> bool {
        true
    }
}

impl Debug for dyn Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target")
    }
}

pub mod texture;
pub use texture::*;

pub mod window;
pub use window::*;
