use crate::{RenderContext, Size, Target, TargetFrame, Texture};
use std::sync::Arc;

pub struct TextureTarget {
    context: Arc<RenderContext>,
    texture: Arc<Texture>,
}

impl TextureTarget {
    pub fn new(context: Arc<RenderContext>, size: Size) -> Self {
        let texture = Arc::new(Texture::create_destination_texture(
            context.as_ref(),
            size.into(),
        ));
        Self {
            context: context.clone(),
            texture,
        }
    }
}

impl Target for TextureTarget {
    fn size(&self) -> Size {
        self.texture.size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        let new_texture =
            Texture::create_destination_texture(self.context.as_ref(), size.into().into());
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

    fn get_image(&self) -> Vec<u8> {
        // Read back pixels from the offscreen texture as a tightly-packed RGBA8 buffer
        let device = &self.context.device;
        let queue = &self.context.queue;
        let w = self.texture.size.width;
        let h = self.texture.size.height;
        let bpp = 4u32; // RGBA8
        let row_bytes = w * bpp;
        let padded_row_bytes =
            wgpu::util::align_to(row_bytes as u64, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64)
                as u32;
        let output_size = (padded_row_bytes as u64 * h as u64) as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TextureTarget readback buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("TextureTarget readback encoder"),
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
        // Block until the GPU work is done and the buffer is mapped
        device.poll(wgpu::PollType::Wait).unwrap();

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::Wait).unwrap();
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
