use crate::{
    app,
    renderer::{target::Dimensions, Renderer},
    resources::sampler::{create_default_sampler, create_sampler, SamplerOptions},
    FragmentColor, Quad,
};
use image::{DynamicImage, GenericImageView};
use std::path::Path;

type Error = Box<dyn std::error::Error>;

const DEFAULT_IMAGE: &str = "default.jpg";
pub(crate) const DEFAULT_IMAGE_SIZE: (u32, u32) = (1200, 1200);

/// Represents a loaded texture in the GPU
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextureId(wgpu::Id<wgpu::Texture>);

/// Default Texture Id
///
/// # Side effects
/// Creates a 1x1 transparent Texture in the GPU
impl Default for TextureId {
    fn default() -> Self {
        let (texture_id, _size) =
            Texture::create_blank_pixel().expect("Failed to create Default pixel!");
        texture_id
    }
}

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
        crate::Quad::from_size(self.size.width, self.size.height)
    }

    fn scaling(&self) -> f32 {
        1.0
    }

    fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}

impl Texture {
    /// Loads a default "Image Not Found" texture
    pub(crate) fn image_not_found() -> Result<(TextureId, Quad), Error> {
        let default = Path::new(&app::ASSETS).join(DEFAULT_IMAGE);

        Self::from_file(default)
    }

    /// Creates a texture from a file
    ///
    /// Returns the Texture Id and the Quad with the size of the loaded texture
    pub fn from_file(path: impl AsRef<Path>) -> Result<(TextureId, Quad), Error> {
        let image = image::open(path)?;
        let size = image.dimensions();
        let texture_id = Self::from_loaded_image(&image)?;

        Ok((texture_id, Quad::from_tuple(size)))
    }

    /// Creates a new texture resource from raw bytes array
    ///
    /// Makes an educated guess about the image format
    /// and automatically detects Width and Height.
    pub fn from_bytes(bytes: &[u8]) -> Result<(TextureId, Quad), Error> {
        let image = image::load_from_memory(bytes)?;
        let size = image.dimensions();
        let texture_id = Self::from_loaded_image(&image)?;

        Ok((texture_id, Quad::from_tuple(size)))
    }

    /// Internal method to create a Texture marked as a destination for rendering
    ///
    /// Unlike the other methods that create a Texture resource in the GPU and
    /// return a TextureId, this will return Self so it can be owned by a Target.
    ///
    /// This method is used internally by the `Target::create_texture()` method.
    pub(crate) fn create_destination_texture(size: wgpu::Extent3d) -> Result<Self, Error> {
        let renderer = FragmentColor::renderer();
        let renderer = if let Ok(renderer) = renderer.try_read() {
            renderer
        } else {
            log::error!("Renderer is locked. Cannot build texture!",);
            return Err("Renderer is locked. Cannot build texture!".into());
        };

        let label = "Render Target Texture";
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::target_texture_descriptor(label, size, format);
        let texture = renderer.device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&renderer.device);

        Ok(Self {
            id: TextureId(texture.global_id()),
            data: texture,
            size,
            view,
            format,
            sampler,
        })
    }

    // We need the DEPTH_FORMAT for when we create the depth stage of
    // the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture
    pub fn create_depth_texture(size: wgpu::Extent3d) -> Result<(TextureId, Quad), Error> {
        let renderer = FragmentColor::renderer();
        let renderer = if let Ok(renderer) = renderer.try_read() {
            renderer
        } else {
            log::error!("Renderer is locked. Cannot build Depth Texture!!",);
            return Err("Renderer is locked. Cannot build Depth Texture!!".into());
        };

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

        Ok((
            renderer.add_texture(texture)?,
            Quad::from_size(size.width, size.height),
        ))
    }

    /// Creates a transparent pixel
    pub fn create_blank_pixel() -> Result<(TextureId, Quad), Error> {
        let renderer = FragmentColor::renderer();
        let renderer = if let Ok(renderer) = renderer.try_read() {
            renderer
        } else {
            log::error!("Renderer is locked. Cannot build Blank Pixel Texture!!",);
            return Err("Renderer is locked. Cannot build Blank Pixel Texture!!".into());
        };

        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::source_texture_descriptor("Default Blank Pixel", size, format);
        let texture = renderer.device.create_texture(&descriptor);
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

        Ok((renderer.add_texture(texture)?, Quad::from_size(1, 1)))
    }

    /// Wraps a wgpu::Texture global_id into a FragmentColor TextureId.
    ///
    /// Expects a loaded wgpu::Texture as input.
    pub(crate) fn id_from(loaded_texture: &wgpu::Texture) -> TextureId {
        TextureId(loaded_texture.global_id())
    }

    //
    // Internal methods _______________________________________________________

    /// Internal method to create a TextureId from a DynamicImage instance.
    ///
    /// The image is already loaded in memory at this point.
    fn from_loaded_image(image: &DynamicImage) -> Result<TextureId, Error> {
        let label = "Source texture";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let descriptor = Self::source_texture_descriptor(label, size, format);

        let renderer = FragmentColor::renderer();
        let renderer = if let Ok(renderer) = renderer.try_read() {
            renderer
        } else {
            return Err("Cannot read Renderer Texture Database. Texture not loaded!".into());
        };

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

    /// Creates a texture descriptor for a Source Texture
    fn source_texture_descriptor(
        label: &str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor {
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

    /// Creates a texture descriptor for a Render Target
    fn target_texture_descriptor(
        label: &str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor {
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
