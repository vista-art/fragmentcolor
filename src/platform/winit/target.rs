use crate::{Renderer, Target, TargetFrame};

pub struct WindowTarget {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowTarget {
    pub fn new(surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration) -> Self {
        Self { surface, config }
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

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&renderer.device, &self.config);
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
