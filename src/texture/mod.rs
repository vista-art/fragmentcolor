use std::sync::Arc;

use lsp_doc::lsp_doc;
use parking_lot::RwLock;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use std::path::Path;

mod sampler;
pub use sampler::*;

pub mod format;
pub use format::*;

use crate::{RenderContext, Size};
use image::{DynamicImage, GenericImageView};

mod error;
pub use error::*;

// Expose Naga image metadata in our public meta struct for now.
use naga::{ImageClass, ImageDimension};

// Unified input type for creating textures (initial Rust subset)
#[derive(Debug, Clone)]
pub enum TextureInput {
    Bytes(Vec<u8>),
    Path(std::path::PathBuf),
    CloneOf(Texture),
    Url(String),
    DynamicImage(DynamicImage),
}

impl From<&[u8]> for TextureInput {
    fn from(v: &[u8]) -> Self {
        TextureInput::Bytes(v.to_vec())
    }
}
impl From<Vec<u8>> for TextureInput {
    fn from(v: Vec<u8>) -> Self {
        TextureInput::Bytes(v)
    }
}
impl From<&Vec<u8>> for TextureInput {
    fn from(v: &Vec<u8>) -> Self {
        TextureInput::Bytes(v.clone())
    }
}
impl From<&std::path::Path> for TextureInput {
    fn from(p: &std::path::Path) -> Self {
        TextureInput::Path(p.to_path_buf())
    }
}
impl From<std::path::PathBuf> for TextureInput {
    fn from(p: std::path::PathBuf) -> Self {
        TextureInput::Path(p)
    }
}
impl From<&std::path::PathBuf> for TextureInput {
    fn from(p: &std::path::PathBuf) -> Self {
        TextureInput::Path(p.clone())
    }
}
impl From<&Texture> for TextureInput {
    fn from(t: &Texture) -> Self {
        TextureInput::CloneOf(t.clone())
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

impl From<u64> for TextureId {
    fn from(value: u64) -> Self {
        TextureId(value)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/texture/texture.md")]
pub struct Texture {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) object: Arc<TextureObject>,
    pub(crate) id: TextureId,
}

impl Texture {
    pub(crate) fn new(
        context: Arc<RenderContext>,
        object: Arc<TextureObject>,
        id: TextureId,
    ) -> Self {
        Self {
            context,
            object,
            id,
        }
    }

    #[lsp_doc("docs/api/core/texture/size.md")]
    pub fn size(&self) -> crate::Size {
        self.object.size()
    }

    #[lsp_doc("docs/api/core/texture/aspect.md")]
    pub fn aspect(&self) -> f32 {
        self.object.aspect()
    }

    #[lsp_doc("docs/api/core/texture/set_sampler_options.md")]
    pub fn set_sampler_options(&self, options: SamplerOptions) {
        self.object
            .set_sampler_options(&self.context.device, options);
    }
}

// Metadata for textures parsed from shader source; users do not construct directly.
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureMeta {
    pub id: TextureId,
    pub dim: ImageDimension,
    pub arrayed: bool,
    pub class: ImageClass,
}

impl TextureMeta {
    pub fn with_id_only(id: TextureId) -> Self {
        TextureMeta {
            id,
            dim: ImageDimension::D2,
            arrayed: false,
            class: ImageClass::Sampled {
                kind: naga::ScalarKind::Float,
                multi: false,
            },
        }
    }
}

// UniformData conversion: allow shader.set("key", &Texture)
impl From<&Texture> for crate::shader::uniform::UniformData {
    fn from(texture: &Texture) -> Self {
        // Provide a placeholder meta; storage.update will merge with shader-parsed meta at set time.
        let meta = TextureMeta::with_id_only(texture.id.clone());
        crate::shader::uniform::UniformData::Texture(meta)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Debug, Clone, Default)]
pub struct TextureOptions {
    pub size: Option<Size>,
    pub format: TextureFormat,
    pub sampler: SamplerOptions,
}

impl From<crate::Size> for TextureOptions {
    fn from(size: crate::Size) -> Self {
        TextureOptions {
            size: Some(size),
            format: TextureFormat::default(),
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<&crate::Size> for TextureOptions {
    fn from(size: &crate::Size) -> Self {
        TextureOptions {
            size: Some(*size),
            format: TextureFormat::default(),
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<TextureFormat> for TextureOptions {
    fn from(format: TextureFormat) -> Self {
        TextureOptions {
            size: None,
            format,
            sampler: SamplerOptions::default(),
        }
    }
}

impl From<&TextureFormat> for TextureOptions {
    fn from(format: &TextureFormat) -> Self {
        TextureOptions {
            size: None,
            format: *format,
            sampler: SamplerOptions::default(),
        }
    }
}

// @TODO move TextureOptions to its own file and implement more conversions
//      reuse the impl from reference macros (look at UniformData for reference)

#[derive(Debug)]
pub(crate) struct TextureObject {
    pub(crate) inner: wgpu::Texture,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) sampler: RwLock<wgpu::Sampler>,
    pub(crate) options: RwLock<SamplerOptions>,
    pub(crate) format: wgpu::TextureFormat,
}

impl TextureObject {
    /// Create a texture with an explicit usage mask (e.g., storage textures)
    pub fn new(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        options: SamplerOptions,
    ) -> Self {
        let descriptor = Self::texture_descriptor("Generic Texture", size, format, usage);
        let inner = context.device.create_texture(&descriptor);
        let size = inner.size();
        let sampler = create_sampler(&context.device, options.clone());
        Self {
            inner,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
        }
    }

    /// Creates a texture from a file
    pub fn from_file(
        context: &RenderContext,
        path: impl AsRef<Path>,
    ) -> Result<Self, TextureError> {
        let image = image::open(path)?;
        Ok(Self::from_loaded_image(context, &image))
    }

    /// Creates a new texture resource from raw encoded image bytes
    /// (PNG/JPEG/HDR, etc.).
    ///
    /// Makes an educated guess about the image format
    /// and automatically detects Width and Height.
    pub fn from_bytes(context: &RenderContext, bytes: &[u8]) -> Result<Self, TextureError> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_loaded_image(context, &image))
    }

    /// Creates a texture from raw pixel bytes with explicit size/format.
    /// The data layout is tightly packed with bytes_per_row = bpp * width.
    pub fn from_raw_bytes(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        data: &[u8],
    ) -> Result<Self, TextureError> {
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let descriptor = Self::texture_descriptor("Raw Texture", size, format, usage);
        let texture = context.device.create_texture(&descriptor);

        let bpp: u32 = match format {
            wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::R8Uint
            | wgpu::TextureFormat::R8Snorm
            | wgpu::TextureFormat::R8Sint => 1,
            wgpu::TextureFormat::Rg8Unorm
            | wgpu::TextureFormat::Rg8Uint
            | wgpu::TextureFormat::Rg8Snorm
            | wgpu::TextureFormat::Rg8Sint => 2,
            wgpu::TextureFormat::R16Float
            | wgpu::TextureFormat::R16Unorm
            | wgpu::TextureFormat::R16Uint
            | wgpu::TextureFormat::R16Snorm
            | wgpu::TextureFormat::R16Sint => 2,
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Rgba8Snorm
            | wgpu::TextureFormat::Rgba8Uint
            | wgpu::TextureFormat::Rgba8Sint
            | wgpu::TextureFormat::Bgra8Unorm => 4,
            wgpu::TextureFormat::Rgba16Uint
            | wgpu::TextureFormat::Rgba16Sint
            | wgpu::TextureFormat::Rgba16Unorm
            | wgpu::TextureFormat::Rgba16Snorm
            | wgpu::TextureFormat::Rgba16Float => 8,
            wgpu::TextureFormat::Rgba32Float
            | wgpu::TextureFormat::Rgba32Uint
            | wgpu::TextureFormat::Rgba32Sint => 16,
            _ => 4,
        };

        // Best-effort guard for buffer size
        let expected = (size.width as usize)
            .saturating_mul(size.height as usize)
            .saturating_mul(size.depth_or_array_layers as usize)
            .saturating_mul(bpp as usize);
        if data.len() < expected {
            return Err(TextureError::CreateTextureError(
                "Insufficient raw data for declared size/format".into(),
            ));
        }

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpp * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let sampler = create_default_sampler(&context.device);
        Ok(Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
        })
    }

    /// Internal method to create a TextureId from a DynamicImage instance.
    ///
    /// The image is already loaded in memory at this point.
    pub(crate) fn from_loaded_image(context: &RenderContext, image: &DynamicImage) -> Self {
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
        let texture = context.device.create_texture(&descriptor);
        let source = image.to_rgba8();
        Self::write_data_to_texture(context, source, &texture, size);

        let sampler = create_default_sampler(&context.device);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
        }
    }

    /// Internal method to create a Texture marked as a destination for rendering
    pub fn create_destination_texture(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> Self {
        let label = "Render Target Texture";
        let descriptor = Self::target_texture_descriptor(label, size, format);
        let texture = context.device.create_texture(&descriptor);
        let sampler = create_default_sampler(&context.device);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
        }
    }

    /// We need the DEPTH_FORMAT for when we create the depth stage of
    /// the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture
    pub fn create_depth_texture(context: &RenderContext, size: wgpu::Extent3d) -> Self {
        let format = Self::DEPTH_FORMAT;
        let descriptor = Self::texture_descriptor(
            "Depth Texture",
            size,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let texture = context.device.create_texture(&descriptor);
        let sampler = create_sampler(
            &context.device,
            SamplerOptions {
                repeat_x: false,
                repeat_y: false,
                smooth: true,
                compare: Some(CompareFunction::LessEqual),
            },
        );

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
        }
    }

    pub fn sampler(&self) -> wgpu::Sampler {
        self.sampler.read().clone()
    }

    pub fn set_sampler_options(&self, device: &wgpu::Device, options: SamplerOptions) {
        let sampler = create_sampler(device, options.clone());
        *self.options.write() = options;
        *self.sampler.write() = sampler;
    }

    pub fn size(&self) -> Size {
        self.size.into()
    }

    pub fn aspect(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.inner
            .create_view(&wgpu::TextureViewDescriptor::default())
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
        context: &RenderContext,
        origin_image: image::RgbaImage,
        target_texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) {
        context.queue.write_texture(
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

#[cfg(test)]
mod tests {
    use super::*;

    // Story: TextureInput conversions from common Rust types produce expected variants.
    #[test]
    fn texture_input_conversions() {
        let bytes: Vec<u8> = vec![1, 2, 3];
        let ti1: TextureInput = (&bytes).into();
        match ti1 {
            TextureInput::Bytes(b) => assert_eq!(b, bytes),
            _ => panic!("expected Bytes"),
        }

        let p = std::path::PathBuf::from("/tmp/img.png");
        let ti2: TextureInput = (&p).into();
        match ti2 {
            TextureInput::Path(pb) => assert_eq!(pb, p),
            _ => panic!("expected Path"),
        }
    }

    // Story: Create a texture from a dynamic image and verify aspect.
    #[test]
    fn dynamic_image_aspect_smoke() {
        pollster::block_on(async move {
            // 2x1 image in memory
            let img = image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_vec(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]).unwrap(),
            );

            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureInput::DynamicImage(img))
                .await
                .expect("create texture from dynamic image");
            assert_eq!(tex.aspect(), 2.0);

            // Change sampler options should not panic
            tex.set_sampler_options(SamplerOptions {
                repeat_x: false,
                repeat_y: false,
                smooth: true,
                compare: None,
            });
        });
    }
}
