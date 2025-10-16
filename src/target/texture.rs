use crate::{RenderContext, Size, Target, TargetFrame, TextureObject};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[cfg_attr(wasm, wasm_bindgen)]
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

    #[lsp_doc("docs/api/targets/target/hidden/get_current_frame.md")]
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        let view = self.texture.create_view();
        let format = self.texture.format();
        Ok(Box::new(TextureFrame { view, format }))
    }

    #[lsp_doc("docs/api/targets/target/get_image.md")]
    fn get_image(&self) -> Vec<u8> {
        // Read back pixels from the offscreen texture as a tightly-packed RGBA8 buffer
        let device = &self.context.device;
        let queue = &self.context.queue;
        let sz = self.texture.size();
        let w = sz.width;
        let h = sz.height;
        let bpp = 4u32; // RGBA8
        let row_bytes = w * bpp;
        let padded_row_bytes =
            wgpu::util::align_to(row_bytes as u64, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64)
                as u32;
        let output_size = (padded_row_bytes as u64 * h as u64) as u64;

        let buffer = {
            let mut pool = self.context.readback_pool.write();
            pool.get(device, output_size)
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureTarget readback encoder (async)"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(h),
                },
            },
            self.texture.size,
        );

        queue.submit(Some(encoder.finish()));

        // Await mapping completion via a oneshot channel future (works on both native and WASM)
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });

        if let Err(e) = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(5)),
        }) {
            log::error!("Device poll error during readback mapping: {:?}", e);
            #[cfg(wasm)]
            {
                log::error!("Ensure the page is cross-origin isolated to enable readbacks.");
            }
            return Vec::new();
        }

        let _ = rx.recv();

        let view = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((w as usize) * (h as usize) * (bpp as usize));
        for y in 0..h as usize {
            let start = y * padded_row_bytes as usize;
            let row = &view[start..start + row_bytes as usize];
            pixels.extend_from_slice(row);
        }
        drop(view);
        buffer.unmap();

        // Convert to RGBA8 if the destination texture uses BGRA8
        match self.texture.format() {
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for px in pixels.chunks_exact_mut(4) {
                    px.swap(0, 2); // BGRA -> RGBA
                }
            }
            _ => {}
        }

        pixels
    }
}

impl TextureTarget {
    /// Obtain a sampleable Texture handle for binding in shaders.
    pub fn texture(&self) -> crate::texture::Texture {
        if let Some(id) = *self.id.read() {
            return crate::texture::Texture::new(self.context.clone(), self.texture.clone(), id);
        }
        let id = self.context.register_texture(self.texture.clone());
        *self.id.write() = Some(id);
        crate::texture::Texture::new(self.context.clone(), self.texture.clone(), id)
    }

    // @TODO deduplicate code with get_image() above
    // This version works with WASM by awaiting the mapping future
    pub async fn get_image_async(&self) -> Vec<u8> {
        // Read back pixels from the offscreen texture as a tightly-packed RGBA8 buffer
        let device = &self.context.device;
        let queue = &self.context.queue;
        let sz = self.texture.size();
        let width = sz.width;
        let height = sz.height;
        let bpp = 4u32; // RGBA8
        let row_bytes = width * bpp;
        let padded_row_bytes =
            wgpu::util::align_to(row_bytes as u64, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64)
                as u32;
        let output_size = (padded_row_bytes as u64 * height as u64) as u64;

        let buffer = {
            let mut pool = self.context.readback_pool.write();
            pool.get(device, output_size)
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureTarget readback encoder (async)"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            self.texture.size,
        );

        queue.submit(Some(encoder.finish()));

        // Await mapping completion via a oneshot channel future (works on both native and WASM)
        let slice = buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = rx.await;

        let view = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((width as usize) * (height as usize) * (bpp as usize));
        for y in 0..height as usize {
            let start = y * padded_row_bytes as usize;
            let row = &view[start..start + row_bytes as usize];
            pixels.extend_from_slice(row);
        }
        drop(view);
        buffer.unmap();

        // Convert to RGBA8 if the destination texture uses BGRA8
        match self.texture.format() {
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                for px in pixels.chunks_exact_mut(4) {
                    px.swap(0, 2); // BGRA -> RGBA
                }
            }
            _ => {}
        }

        pixels
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
