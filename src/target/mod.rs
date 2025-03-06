use crate::Renderer;
use std::fmt::{Debug, Formatter};

pub mod texture;
pub mod winit;

pub use texture::*;
pub use winit::*;

pub trait Target {
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d);
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError>;
}

pub trait TargetFrame {
    fn view(&self) -> &wgpu::TextureView;
    fn present(self: Box<Self>);
}

impl Debug for dyn Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Target")
    }
}
