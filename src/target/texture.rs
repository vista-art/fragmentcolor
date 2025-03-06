use std::sync::Arc;

use crate::{Renderer, Target, TargetFrame, Texture};

pub struct TextureTarget {
    texture: Arc<Texture>,
}

impl Target for TextureTarget {
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        let new_texture = Texture::create_destination_texture(renderer, size);
        self.texture = Arc::new(new_texture);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        let view = self
            .texture
            .inner
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Box::new(TextureFrame { view }))
    }
}

struct TextureFrame {
    view: wgpu::TextureView,
}

impl TargetFrame for TextureFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn present(self: Box<Self>) {
        // No-op for textures
    }
}
