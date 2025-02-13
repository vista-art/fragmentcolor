use crate::{Renderer, Texture};
use std::sync::Arc;

#[derive(Debug)]
pub enum Target {
    Texture(TextureTarget),
    Surface(SurfaceTarget),
}

#[derive(Debug)]
pub struct TextureTarget {
    pub texture: Arc<Texture>,
}

#[derive(Debug)]
pub struct SurfaceTarget {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Debug)]
pub(crate) struct PresentationSurface {
    surface_texture: Option<wgpu::SurfaceTexture>,
    pub _view: wgpu::TextureView,
}

impl PresentationSurface {
    pub fn present(self) {
        if let Some(surface_texture) = self.surface_texture {
            surface_texture.present();
        }
    }
}

impl Target {
    pub fn from_texture(texture: Texture) -> Self {
        Self::Texture(TextureTarget {
            texture: Arc::new(texture),
        })
    }

    // @TODO platform-specific initialization, i.e. from canvas or native window pointer
    pub fn from_surface(
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self::Surface(SurfaceTarget { surface, config })
    }

    pub fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        match self {
            Self::Texture(target) => {
                let texture = Texture::create_destination_texture(renderer, size);
                target.texture = Arc::new(texture);
            }
            Self::Surface(target) => {
                let surface = &target.surface;
                let config = &target.config;
                surface.configure(&renderer.device, config);
            }
        }
    }

    pub(crate) fn get_current_texture(&self) -> Result<PresentationSurface, wgpu::SurfaceError> {
        match self {
            Self::Texture(target) => Ok(PresentationSurface {
                surface_texture: None,
                _view: target.texture.inner.create_view(&Default::default()),
            }),
            Self::Surface(target) => {
                let surface_texture = target.surface.get_current_texture()?;
                let _view = surface_texture.texture.create_view(&Default::default());
                Ok(PresentationSurface {
                    surface_texture: Some(surface_texture),
                    _view,
                })
            }
        }
    }
}
