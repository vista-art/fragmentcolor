use crate::{RenderContext, Size, Target, TargetFrame, TextureObject};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[lsp_doc("docs/api/targets/texture_target/texture_target.md")]
pub struct TextureTarget {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) texture: Arc<TextureObject>,
    pub(crate) id: Arc<parking_lot::RwLock<Option<crate::texture::TextureId>>>,
}

crate::impl_fc_kind!(TextureTarget, "TextureTarget");

impl TextureTarget {
    pub(crate) fn new(
        context: Arc<RenderContext>,
        size: Size,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = Arc::new(TextureObject::create_destination_texture(
            context.as_ref(),
            size.into(),
            format,
        ));
        Self {
            context: context.clone(),
            texture,
            id: Arc::new(parking_lot::RwLock::new(None)),
        }
    }
}

impl crate::target::TargetInternal for TextureTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        let view = self.texture.create_view();
        let format = self.texture.format();
        Ok(Box::new(TextureFrame { view, format }))
    }
}

impl Target for TextureTarget {
    #[lsp_doc("docs/api/targets/target/size.md")]
    fn size(&self) -> Size {
        self.texture.size()
    }

    #[lsp_doc("docs/api/targets/target/resize.md")]
    fn resize(&mut self, size: impl Into<Size>) {
        let new_texture = TextureObject::create_destination_texture(
            self.context.as_ref(),
            size.into().into(),
            self.texture.format(),
        );
        self.texture = Arc::new(new_texture);
    }

    /// Read back the offscreen texture contents as packed RGBA8 bytes
    /// (row-major, top-left origin). Mirrors `Texture::get_image()` — both
    /// are async-only so the readback path is identical on every backend
    /// (browser `fetch`/`mapAsync`, native `device.poll(Wait)`, etc.).
    #[lsp_doc("docs/api/targets/target/get_image.md")]
    async fn get_image(&self) -> Vec<u8> {
        let mut pixels = crate::texture::read_pixels(&self.context, &self.texture)
            .await
            .unwrap_or_else(|e| {
                log::error!("TextureTarget::get_image failed: {:?}", e);
                Vec::new()
            });
        swap_bgra_to_rgba(&mut pixels, self.texture.format());
        pixels
    }
}

impl TextureTarget {
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture(&self) -> crate::texture::Texture {
        if let Some(id) = *self.id.read() {
            return crate::texture::Texture::new(self.context.clone(), self.texture.clone(), id);
        }
        let id = self.context.register_texture(self.texture.clone());
        *self.id.write() = Some(id);
        crate::texture::Texture::new(self.context.clone(), self.texture.clone(), id)
    }
}

fn swap_bgra_to_rgba(pixels: &mut [u8], format: wgpu::TextureFormat) {
    if matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
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

    fn present(self: Box<Self>, _queue: &wgpu::Queue) {
        // No-op for textures
    }
}
