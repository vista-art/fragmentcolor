use crate::{Renderer, Target, TargetFrame};
use pyo3::prelude::*;

#[pyclass]
pub struct PyWindowTarget {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

#[pyclass]
pub struct PyWindowFrame {
    surface_texture: wgpu::SurfaceTexture,
    format: wgpu::TextureFormat,
    view: wgpu::TextureView,
}

impl Target for PyWindowTarget {
    fn size(&self) -> wgpu::Extent3d {
        unimplemented!()
    }

    fn resize(&mut self, _renderer: &Renderer, _size: wgpu::Extent3d) {
        unimplemented!()
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        unimplemented!()
    }
}

impl TargetFrame for PyWindowFrame {
    fn view(&self) -> &wgpu::TextureView {
        unimplemented!()
    }

    fn format(&self) -> wgpu::TextureFormat {
        unimplemented!()
    }

    fn present(self: Box<Self>) {
        unimplemented!()
    }
}
