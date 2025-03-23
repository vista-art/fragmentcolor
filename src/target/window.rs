use crate::{RenderContext, Target, TargetFrame};
use std::sync::Arc;

pub struct WindowTarget {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl WindowTarget {
    pub fn new(
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            context,
            surface,
            config,
        }
    }
}

impl Target for WindowTarget {
    fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        }
    }

    fn resize(&mut self, size: wgpu::Extent3d) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.context.device, &self.config);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Box::new(WindowFrame {
            surface_texture,
            format: self.config.format,
            view,
        }))
    }
}

struct WindowFrame {
    surface_texture: wgpu::SurfaceTexture,
    format: wgpu::TextureFormat,
    view: wgpu::TextureView,
}

impl TargetFrame for WindowFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn present(self: Box<Self>) {
        self.surface_texture.present();
    }
}

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
pub struct CanvasTarget(WindowTarget);

impl CanvasTarget {
    pub fn new(
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self(WindowTarget::new(context, surface, config))
    }
}

impl Target for CanvasTarget {
    fn size(&self) -> wgpu::Extent3d {
        self.0.size()
    }

    fn resize(&mut self, size: wgpu::Extent3d) {
        self.0.resize(size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
}
