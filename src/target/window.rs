use crate::{RenderContext, Size, Target, TargetFrame};
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
    fn size(&self) -> Size {
        [self.config.width, self.config.height].into()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        let size = size.into();
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
