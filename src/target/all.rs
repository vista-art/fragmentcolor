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

#[cfg(test)]
mod tests {
    use super::*;

    // Story: RenderTarget delegates to underlying texture target for size/resize/frame/image.
    #[test]
    fn delegates_for_texture_variant() {
        let r = crate::Renderer::new();
        let rt = pollster::block_on(r.create_texture_target([8, 6])).expect("tex");
        let mut any = RenderTarget::from(rt);

        let s = any.size();
        assert_eq!([s.width, s.height], [8, 6]);

        any.resize([4, 4]);
        let s2 = any.size();
        assert_eq!([s2.width, s2.height], [4, 4]);

        let fr = any.get_current_frame().expect("frame");
        let _fmt = fr.format();
        let img = any.get_image();
        assert_eq!(img.len() as u32, 4 * 4 * 4);
    }

    // Story: RenderTarget created from headless window behaves like texture-backed variant.
    #[test]
    fn delegates_for_window_variant_headless_fallback() {
        let r = crate::Renderer::new();
        let headless = crate::headless_window([10, 12]);
        let target = pollster::block_on(r.create_target(headless)).expect("target");

        let size = target.size();
        assert_eq!([size.width, size.height], [10, 12]);
        let image = target.get_image();
        assert_eq!(image.len() as u32, 10 * 12 * 4);
    }
}
