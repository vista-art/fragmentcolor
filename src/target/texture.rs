use crate::core::Start;
use crate::texture::{Registration, TextureError};
use crate::{RenderContext, Size, Target, TargetFrame, TextureObject};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
#[cfg_attr(wasm, wasm_bindgen)]
#[lsp_doc("docs/api/targets/texture_target/texture_target.md")]
pub struct TextureTarget {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) registration: Arc<Registration>,
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
            size.to_extent(),
            format,
        ));
        let registration = Registration::create(&context, texture);
        Self {
            context,
            registration,
        }
    }
}

impl crate::target::FrameSource for TextureTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        let texture = self.registration.object();
        let view = texture.create_view();
        let format = texture.format();
        let size = texture.size;
        Ok(Box::new(TextureFrame { view, format, size }))
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        Some(self.context.clone())
    }
}

impl Target for TextureTarget {
    #[lsp_doc("docs/api/targets/target/size.md")]
    fn size(&self) -> Size {
        self.registration.object().size()
    }

    #[lsp_doc("docs/api/targets/target/resize.md")]
    fn resize(&mut self, size: impl Into<Size>) -> Result<(), TextureError> {
        // The swap reaches every pass attachment, shader binding, clone and
        // texture() handle that holds the registration.
        crate::texture::resize(&self.context, &self.registration, size.into())
    }

    /// Read back the offscreen texture contents as packed RGBA8 bytes
    /// (row-major, top-left origin). Mirrors `Texture::get_image()` — both
    /// are async-only so the readback path is identical on every backend
    /// (browser `fetch`/`mapAsync`, native `device.poll(Wait)`, etc.).
    #[lsp_doc("docs/api/targets/target/get_image.md")]
    async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        self.read_image_from(Start::now(Some(&self.context))).await
    }
}

impl TextureTarget {
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture(&self) -> crate::texture::Texture {
        crate::texture::Texture::new(self.context.clone(), self.registration.clone())
    }

    /// Reads the target as RGBA8 bytes once the frames rendered before `start`
    /// have reported their GPU errors.
    pub(crate) async fn read_image_from(&self, start: Start) -> Result<Vec<u8>, TextureError> {
        let texture = self.registration.object();
        let mut pixels = crate::texture::read_pixels_from(&self.context, &texture, start).await?;
        crate::texture::swap_bgra(&mut pixels, texture.format());
        Ok(pixels)
    }
}

struct TextureFrame {
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
}

impl TargetFrame for TextureFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn size(&self) -> wgpu::Extent3d {
        self.size
    }

    fn present(self: Box<Self>, _queue: &wgpu::Queue) {
        // No-op for textures
    }
}
