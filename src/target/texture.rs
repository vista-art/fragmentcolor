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
            size.into(),
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
        Ok(Box::new(TextureFrame { view, format }))
    }
}

impl Target for TextureTarget {
    #[lsp_doc("docs/api/targets/target/size.md")]
    fn size(&self) -> Size {
        self.registration.object().size()
    }

    #[lsp_doc("docs/api/targets/target/resize.md")]
    fn resize(&mut self, size: impl Into<Size>) {
        let extent = size.into().into();
        // Swapping the registration's texture repoints every pass attachment,
        // shader binding, clone and texture() handle that holds it.
        let resized = self.registration.replace(|current| {
            let format = current.format();
            let texture = TextureObject::create_destination_texture(&self.context, extent, format);
            Ok(Some(texture))
        });
        if let Err(error) = resized {
            log::warn!("TextureTarget::resize kept the old texture: {error}");
        }
    }

    /// Read back the offscreen texture contents as packed RGBA8 bytes
    /// (row-major, top-left origin). Mirrors `Texture::get_image()` — both
    /// are async-only so the readback path is identical on every backend
    /// (browser `fetch`/`mapAsync`, native `device.poll(Wait)`, etc.).
    #[lsp_doc("docs/api/targets/target/get_image.md")]
    async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        let texture = self.registration.object();
        let mut pixels = crate::texture::read_pixels(&self.context, &texture).await?;
        swap_bgra_to_rgba(&mut pixels, texture.format());
        Ok(pixels)
    }
}

impl TextureTarget {
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture(&self) -> crate::texture::Texture {
        crate::texture::Texture::new(self.context.clone(), self.registration.clone())
    }
}

fn swap_bgra_to_rgba(pixels: &mut [u8], format: wgpu::TextureFormat) {
    if matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        for px in pixels.as_chunks_mut::<4>().0 {
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
