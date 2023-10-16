use crate::renderer::{
    resources::sampler::{create_default_sampler, create_sampler, SamplerOptions},
    Renderer,
};
use image::{GenericImageView, RgbaImage};

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureRef(pub u32);

#[derive(Debug)]
pub struct Texture {
    pub data: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(renderer: &crate::renderer::Renderer, bytes: &[u8]) -> Result<Self, Error> {
        let image = image::load_from_memory(bytes)?;
        Self::from_image(renderer, &image)
    }

    pub fn from_image(
        renderer: &crate::renderer::Renderer,
        image: &image::DynamicImage,
    ) -> Result<Self, Error> {
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let label = "Source texture from image";
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::source_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);

        let source = image.to_rgba8();
        Self::write_data_to_texture(&renderer, source, &texture, size);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        Ok(Self {
            data: texture,
            size,
            view,
            format,
            sampler,
        })
    }

    pub fn from_wgpu_texture(renderer: &crate::renderer::Renderer, texture: wgpu::Texture) -> Self {
        let size = texture.size();
        let format = texture.format();
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        Self {
            data: texture,
            size,
            view,
            format,
            sampler,
        }
    }

    pub fn write_data_to_texture(
        renderer: &crate::renderer::Renderer,
        origin_image: RgbaImage,
        target_texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) {
        renderer.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            // The actual pixel data
            &origin_image,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        )
    }

    pub fn create_target_texture(renderer: &Renderer, size: wgpu::Extent3d) -> Self {
        let label = "Render target texture";
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::target_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        Self {
            data: texture,
            size,
            view,
            format,
            sampler,
        }
    }

    // We need the DEPTH_FORMAT for when we create the depth stage of
    // the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth_texture(
        renderer: &crate::renderer::Renderer,
        size: wgpu::Extent3d,
    ) -> Self {
        let label = "Depth texture";
        let format = Self::DEPTH_FORMAT;
        let descriptor = Self::source_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_sampler(
            &renderer.device,
            SamplerOptions {
                repeat_x: false,
                repeat_y: false,
                smooth: true,
                compare: Some(wgpu::CompareFunction::LessEqual),
            },
        );

        Self {
            data: texture,
            size,
            view,
            format,
            sampler,
        }
    }

    fn source_descriptor<'a>(
        label: &'a str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'a> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }
    }

    fn target_descriptor<'a>(
        label: &'a str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'a> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }
}
