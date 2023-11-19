use crate::{
    app::error::READ_LOCK_ERROR,
    renderer::{target::Dimensions, Renderer},
    resources::sampler::{create_default_sampler, create_sampler, SamplerOptions},
    PLRender,
};
use image::{DynamicImage, GenericImageView};
use std::path::Path;

type Error = Box<dyn std::error::Error>;

/// Represents a loaded texture in the GPU
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureId(wgpu::Id<wgpu::Texture>);

#[derive(Debug)]
pub struct Texture {
    pub id: TextureId,
    pub data: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub sampler: wgpu::Sampler,
}

impl Dimensions for Texture {
    fn size(&self) -> crate::Quad {
        crate::Quad::from_dimensions(self.size.width, self.size.height)
    }

    fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}

impl Texture {
    /// Creates a texture from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<TextureId, Error> {
        let image = image::open(path)?;
        Ok(Self::from_image(&image))
    }

    /// Creates a new texture resource from raw bytes array
    ///
    /// Makes an educated guess about the image format
    /// and automatically detects Width and Height.
    pub fn from_bytes(bytes: &[u8]) -> Result<TextureId, Error> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_image(&image))
    }

    /// Creates a Texture from a DynamicImage instance.
    ///
    /// The image is already loaded in memory at this point.
    pub fn from_image(image: &DynamicImage) -> TextureId {
        let label = "Source texture";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::source_texture_descriptor(label, size, format);

        let renderer = PLRender::renderer().read().expect(READ_LOCK_ERROR);
        let texture = renderer.device.create_texture(&descriptor);

        let source = image.to_rgba8();
        Self::write_data_to_texture(&renderer, source, &texture, size);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        let texture = Self {
            id: TextureId(texture.global_id()),
            data: texture,
            size,
            view,
            format,
            sampler,
        };

        renderer.add_texture(texture)
    }

    /// Creates a texture marked as a destination for rendering
    ///
    /// Unlike the other methods, it doesn't load itself to the Renderer resources
    /// because it is a RenderTarget.
    pub fn create_target_texture(size: wgpu::Extent3d) -> Self {
        let renderer = PLRender::renderer().read().expect(READ_LOCK_ERROR);

        let label = "Render Target Texture";
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::target_texture_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        Self {
            id: TextureId(texture.global_id()),
            data: texture,
            size,
            view,
            format,
            sampler,
        }

        // @TODO it's probably a good idea to create Renderer.add_texture_target()
        //       which will create a TextureTarget and add it to the Renderer targets
    }

    // We need the DEPTH_FORMAT for when we create the depth stage of
    // the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth_texture(size: wgpu::Extent3d) -> TextureId {
        let renderer = PLRender::renderer().read().expect(READ_LOCK_ERROR);

        let label = "Depth Texture";
        let format = Self::DEPTH_FORMAT;
        let descriptor = Self::source_texture_descriptor(label, size, format);
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

        let texture = Self {
            id: TextureId(texture.global_id()),
            data: texture,
            size,
            view,
            format,
            sampler,
        };
        renderer.add_texture(texture)
    }

    fn source_texture_descriptor<'a>(
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

    fn target_texture_descriptor<'a>(
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

    /// Writes pixel data to a texture
    fn write_data_to_texture(
        renderer: &Renderer,
        origin_image: image::RgbaImage,
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
}
