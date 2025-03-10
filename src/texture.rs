use std::path::Path;

use crate::Renderer;
use image::{DynamicImage, GenericImageView};

use crate::sampler::{create_default_sampler, create_sampler, SamplerOptions};

type Error = Box<dyn std::error::Error>; // @TODO tech debt: create proper error types

#[derive(Debug)]
pub struct Texture {
    pub inner: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
}

impl Texture {
    pub fn new(
        renderer: &Renderer,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        options: SamplerOptions,
    ) -> Self {
        let label = "Generic Texture";
        let descriptor = Self::texture_descriptor(
            label,
            size,
            format,
            // Allows all usages by default
            wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let inner = renderer.device.create_texture(&descriptor);
        let size = inner.size();
        let sampler = create_sampler(&renderer.device, options);

        Self {
            inner,
            size,
            sampler,
            format,
        }
    }

    // @TODO this should be behind a feature flag
    /// Creates a texture from a file
    pub fn from_file(renderer: &Renderer, path: impl AsRef<Path>) -> Result<Self, Error> {
        let image = image::open(path)?;
        Ok(Self::from_loaded_image(renderer, &image))
    }

    /// Creates a new texture resource from raw bytes array
    ///
    /// Makes an educated guess about the image format
    /// and automatically detects Width and Height.
    pub fn from_bytes(renderer: &Renderer, bytes: &[u8]) -> Result<Self, Error> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_loaded_image(renderer, &image))
    }

    /// Internal method to create a TextureId from a DynamicImage instance.
    ///
    /// The image is already loaded in memory at this point.
    fn from_loaded_image(renderer: &Renderer, image: &DynamicImage) -> Self {
        let label = "Source Texture from Loaded Image";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = image.color();
        let format = match format {
            image::ColorType::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            image::ColorType::L8 => wgpu::TextureFormat::R8Unorm,
            image::ColorType::La8 => wgpu::TextureFormat::Rg8Unorm,
            image::ColorType::Rgb8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            image::ColorType::L16 => wgpu::TextureFormat::R16Unorm,
            image::ColorType::La16 => wgpu::TextureFormat::Rg16Unorm,
            image::ColorType::Rgb16 => wgpu::TextureFormat::Rgba16Unorm,
            image::ColorType::Rgba16 => wgpu::TextureFormat::Rgba16Unorm,
            image::ColorType::Rgb32F => wgpu::TextureFormat::Rgba32Float,
            image::ColorType::Rgba32F => wgpu::TextureFormat::Rgba32Float,
            _ => wgpu::TextureFormat::Rgba8UnormSrgb,
        };
        let descriptor = Self::source_texture_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let source = image.to_rgba8();
        Self::write_data_to_texture(renderer, source, &texture, size);

        let sampler = create_default_sampler(&renderer.device);

        Self {
            inner: texture,
            size,
            sampler,
            format,
        }
    }

    /// Internal method to create a Texture marked as a destination for rendering
    ///
    /// Unlike the other methods that create a Texture resource in the GPU and
    /// return a TextureId, this will return Self so it can be owned by a Target.
    ///
    /// This method is used internally by the `Target::create_texture()` method.
    pub fn create_destination_texture(renderer: &Renderer, size: wgpu::Extent3d) -> Self {
        let label = "Render Target Texture";
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::target_texture_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let sampler = create_default_sampler(&renderer.device);

        Self {
            inner: texture,
            size,
            sampler,
            format,
        }
    }

    // We need the DEPTH_FORMAT for when we create the depth stage of
    // the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture
    pub fn create_depth_texture(renderer: &Renderer, size: wgpu::Extent3d) -> Self {
        let format = Self::DEPTH_FORMAT;
        let descriptor = Self::target_texture_descriptor("Depth Texture", size, format);
        let texture = renderer.device.create_texture(&descriptor);
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
            inner: texture,
            size,
            sampler,
            format,
        }
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    /// Creates a texture descriptor for a Source Texture
    fn source_texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'static> {
        Self::texture_descriptor(
            label,
            size,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        )
    }

    /// Creates a texture descriptor for a Target Texture
    fn target_texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'static> {
        Self::texture_descriptor(
            label,
            size,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    /// Creates a texture descriptor
    fn texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: match size.depth_or_array_layers {
                1 => wgpu::TextureDimension::D2,
                _ => wgpu::TextureDimension::D3,
            },
            format,
            view_formats: &[],
            usage,
        }
    }

    /// Writes pixel data to a texture
    fn write_data_to_texture(
        renderer: &Renderer,
        origin_image: image::RgbaImage,
        target_texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) {
        renderer.queue.write_texture(
            // Tells wgpu where to copy the pixel data from
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            // The actual pixel data
            &origin_image,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width), // @TODO: handle other formats
                rows_per_image: Some(size.height),
            },
            size,
        )
    }
}
