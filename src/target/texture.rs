use crate::{RenderContext, Target, TargetFrame, Texture};
use std::sync::Arc;

pub struct TextureTarget {
    context: Arc<RenderContext>,
    texture: Arc<Texture>,
}

impl TextureTarget {
    pub fn new(context: Arc<RenderContext>, size: wgpu::Extent3d) -> Self {
        let texture = Arc::new(Texture::create_destination_texture(context.as_ref(), size));
        Self {
            context: context.clone(),
            texture,
        }
    }
}

impl Target for TextureTarget {
    fn size(&self) -> wgpu::Extent3d {
        self.texture.size()
    }

    fn resize(&mut self, size: wgpu::Extent3d) {
        let new_texture = Texture::create_destination_texture(self.context.as_ref(), size);
        self.texture = Arc::new(new_texture);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        let view = self
            .texture
            .inner
            .create_view(&wgpu::TextureViewDescriptor::default());
        let format = self.texture.format;
        Ok(Box::new(TextureFrame { view, format }))
    }
}

struct TextureFrame {
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
}

impl TargetFrame for TextureFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn present(self: Box<Self>) {
        // No-op for textures
    }
}
