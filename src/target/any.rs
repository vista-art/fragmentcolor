use crate::{Size, Target, TargetFrame, TextureTarget, WindowTarget};

pub enum RenderTarget {
    Window(WindowTarget),
    Texture(TextureTarget),
}

impl std::fmt::Debug for RenderTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderTarget::Window(_) => write!(f, "RenderTarget::Window"),
            RenderTarget::Texture(_) => write!(f, "RenderTarget::Texture"),
        }
    }
}

impl From<WindowTarget> for RenderTarget {
    fn from(w: WindowTarget) -> Self {
        RenderTarget::Window(w)
    }
}

impl From<TextureTarget> for RenderTarget {
    fn from(t: TextureTarget) -> Self {
        RenderTarget::Texture(t)
    }
}

impl Target for RenderTarget {
    fn size(&self) -> Size {
        match self {
            RenderTarget::Window(w) => w.size(),
            RenderTarget::Texture(t) => t.size(),
        }
    }

    fn resize(&mut self, size: impl Into<Size>) {
        match self {
            RenderTarget::Window(w) => w.resize(size),
            RenderTarget::Texture(t) => t.resize(size),
        }
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        match self {
            RenderTarget::Window(w) => w.get_current_frame(),
            RenderTarget::Texture(t) => t.get_current_frame(),
        }
    }

    fn get_image(&self) -> Vec<u8> {
        match self {
            RenderTarget::Window(w) => w.get_image(),
            RenderTarget::Texture(t) => t.get_image(),
        }
    }
}
