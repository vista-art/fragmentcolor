use std::sync::Arc;

use crate::{Renderer, Target, TargetFrame, Texture};

pub struct TextureTarget {
    texture: Arc<Texture>,
}

impl Target for TextureTarget {
    fn size(&self) -> wgpu::Extent3d {
        self.texture.size().into()
    }

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        let new_texture = Texture::create_destination_texture(renderer, size);
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
