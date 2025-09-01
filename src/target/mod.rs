mod texture;
pub use texture::*;

mod window;
pub use window::*;

use crate::size::Size;

pub trait Target {
    fn size(&self) -> Size;
    fn resize(&mut self, size: impl Into<Size>);
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
