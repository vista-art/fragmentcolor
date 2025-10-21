use crate::shader::uniform::UniformData;
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

mod platform;

use crate::{RenderContext, Size};
use image::{DynamicImage, GenericImageView};

mod error;
pub use error::*;

mod options;
pub use options::*;

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

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureInput {
    type Error = crate::texture::error::TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer, Uint8Array, Uint8ClampedArray};
        use wasm_bindgen::JsCast;

        // Case: Uint8Array (fast path)
        if let Some(u8a) = value.dyn_ref::<Uint8Array>() {
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: Uint8ClampedArray (ImageData.data)
        if let Some(u8c) = value.dyn_ref::<Uint8ClampedArray>() {
            let mut bytes = vec![0u8; u8c.length() as usize];
            u8c.copy_to(&mut bytes[..]);
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: ArrayBuffer
        if let Some(buf) = value.dyn_ref::<ArrayBuffer>() {
            // Guard against detached buffers via byte_length() == 0; safe fallback for empty too.
            if buf.byte_length() == 0 {
                return Ok(TextureInput::Bytes(Vec::new()));
            }
            let u8a = Uint8Array::new(buf);
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: ImageData -> use its backing data (Clamped<Vec<u8>> on wasm)
        if let Some(image_data) = value.dyn_ref::<web_sys::ImageData>() {
            let data = image_data.data();
            let bytes = data.0;
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: HTMLCanvasElement -> raw pixel bytes via 2D context
        if let Some(canvas) = value.dyn_ref::<web_sys::HtmlCanvasElement>()
            && let Ok(Some(ctx_js)) = canvas.get_context("2d")
            && let Ok(ctx) = ctx_js.dyn_into::<web_sys::CanvasRenderingContext2d>()
        {
            let width = canvas.width() as f64;
            let height = canvas.height() as f64;
            let img = ctx.get_image_data(0.0, 0.0, width, height)?;
            let data = img.data();
            let bytes = data.0;
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: OffscreenCanvas -> raw pixel bytes via 2D context
        if let Some(canvas) = value.dyn_ref::<web_sys::OffscreenCanvas>()
            && let Ok(Some(ctx_js)) = canvas.get_context("2d")
            && let Ok(ctx) = ctx_js.dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
        {
            let width = canvas.width() as f64;
            let height = canvas.height() as f64;
            let img = ctx.get_image_data(0.0, 0.0, width, height)?;
            let data = img.data();
            let bytes = data.0;
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: HTMLImageElement -> pass through as URL
        if let Some(img) = value.dyn_ref::<web_sys::HtmlImageElement>() {
            return Ok(TextureInput::Url(img.src()));
        }

        // Case: Plain JS Array of numbers
        if Array::is_array(value) {
            let arr = Array::from(value);
            let len = arr.length();
            let mut bytes = Vec::with_capacity(len as usize);
            for i in 0..len {
                let n = arr.get(i).as_f64().unwrap_or(0.0);
                let n = if n.is_nan() { 0.0 } else { n };
                let b = n.max(0.0).min(255.0) as u8;
                bytes.push(b);
            }
            return Ok(TextureInput::Bytes(bytes));
        }

        // Case: String -> selector or URL
        if let Some(s) = value.as_string() {
            let url = if s.starts_with('#') || s.starts_with('.') {
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Ok(Some(elem)) = doc.query_selector(&s) {
                        if let Some(img) = elem.dyn_ref::<web_sys::HtmlImageElement>() {
                            img.src()
                        } else if let Some(canvas) = elem.dyn_ref::<web_sys::HtmlCanvasElement>() {
                            canvas.to_data_url().unwrap_or(s.clone())
                        } else {
                            s.clone()
                        }
                    } else {
                        s.clone()
                    }
                } else {
                    s.clone()
                }
            } else {
                s.clone()
            };
            return Ok(TextureInput::Url(url));
        }

        Err(crate::texture::error::TextureError::Error(
            "Unsupported input for TextureInput".into(),
        ))
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

impl From<u64> for TextureId {
    fn from(value: u64) -> Self {
        TextureId(value)
    }
}

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

crate::impl_fc_kind!(Texture, "Texture");

impl Texture {
    /// Creates a new Texture from a RenderContext.
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

    /// Return the stable TextureId for this texture.
    /// The id is valid within the Renderer that created it.
    pub(crate) fn id(&self) -> &TextureId {
        &self.id
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
impl From<&Texture> for UniformData {
    fn from(texture: &Texture) -> Self {
        // Provide a placeholder meta; storage.update will merge with shader-parsed meta at set time.
        let meta = TextureMeta::with_id_only(texture.id);
        UniformData::Texture(meta)
    }
}

#[derive(Debug)]
pub(crate) struct TextureObject {
    pub(crate) inner: wgpu::Texture,
    pub(crate) size: wgpu::Extent3d,
    pub(crate) sampler: RwLock<wgpu::Sampler>,
    pub(crate) options: RwLock<SamplerOptions>,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) usage: wgpu::TextureUsages,
}

// platform-specific bindings live under texture/platform/{python,web}.rs

impl TextureObject {
    /// Create a texture with an explicit usage mask (e.g., storage textures)
    pub fn new(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        options: SamplerOptions,
    ) -> Self {
        let descriptor = Self::texture_descriptor("Generic Texture", size, format, usage, 1, 1); // @TODO mip_count, sample_count
        let inner = context.device.create_texture(&descriptor);
        let size = inner.size();
        let sampler = create_sampler(&context.device, options);
        Self {
            inner,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
            usage,
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
        let descriptor = Self::texture_descriptor("Raw Texture", size, format, usage, 1, 1); // @TODO mip_count, sample_count
        let texture = context.device.create_texture(&descriptor);
        let bpp = bytes_per_pixel(format);

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
            usage,
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    /// We need the DEPTH_FORMAT for when we create the depth stage of
    /// the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a depth texture inheriting the current renderer sample count.
    pub fn create_depth_texture(context: &RenderContext, size: wgpu::Extent3d) -> Self {
        let sc = context.sample_count();
        Self::create_depth_texture_with_count(context, size, sc)
    }

    /// Creates a depth texture with an explicit MSAA sample count.
    pub fn create_depth_texture_with_count(
        context: &RenderContext,
        size: wgpu::Extent3d,
        sample_count: u32,
    ) -> Self {
        let format = Self::DEPTH_FORMAT;
        let descriptor = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: sample_count.max(1),
            dimension: match size.depth_or_array_layers {
                1 => wgpu::TextureDimension::D2,
                _ => wgpu::TextureDimension::D3,
            },
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    pub fn sampler(&self) -> wgpu::Sampler {
        self.sampler.read().clone()
    }

    pub fn set_sampler_options(&self, device: &wgpu::Device, options: SamplerOptions) {
        let sampler = create_sampler(device, options);
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
        let sample_count = 1;
        let mip_level_count = 1;
        Self::texture_descriptor(
            label,
            size,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            sample_count,
            mip_level_count,
        )
    }

    /// Creates a texture descriptor for a Target Texture
    fn target_texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'static> {
        let sample_count = 1;
        let mip_level_count = 1;
        Self::texture_descriptor(
            label,
            size,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count,
            mip_level_count,
        )
    }

    /// Creates a texture descriptor
    fn texture_descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        sample_count: u32,
        mip_level_count: u32,
    ) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count,
            sample_count,
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
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        )
    }
}

fn bytes_per_pixel(format: wgpu::TextureFormat) -> u32 {
    match format {
        wgpu::TextureFormat::R8Unorm
        | wgpu::TextureFormat::R8Uint
        | wgpu::TextureFormat::R8Sint
        | wgpu::TextureFormat::R8Snorm
        | wgpu::TextureFormat::Stencil8 => 1,
        wgpu::TextureFormat::Rg8Uint
        | wgpu::TextureFormat::Rg8Sint
        | wgpu::TextureFormat::R16Uint
        | wgpu::TextureFormat::R16Sint
        | wgpu::TextureFormat::Rg16Uint
        | wgpu::TextureFormat::Rg16Sint
        | wgpu::TextureFormat::R16Float
        | wgpu::TextureFormat::Rg8Unorm
        | wgpu::TextureFormat::Rg8Snorm
        | wgpu::TextureFormat::R16Unorm
        | wgpu::TextureFormat::R16Snorm
        | wgpu::TextureFormat::Rg16Unorm
        | wgpu::TextureFormat::Rg16Snorm
        | wgpu::TextureFormat::Depth16Unorm => 2,
        wgpu::TextureFormat::R32Uint
        | wgpu::TextureFormat::R32Sint
        | wgpu::TextureFormat::R32Float
        | wgpu::TextureFormat::Rgba8Uint
        | wgpu::TextureFormat::Rgba8Sint
        | wgpu::TextureFormat::Rg16Float
        | wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8Snorm
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Rgb10a2Uint
        | wgpu::TextureFormat::Depth24Plus
        | wgpu::TextureFormat::Rgb9e5Ufloat
        | wgpu::TextureFormat::Depth32Float
        | wgpu::TextureFormat::Rgb10a2Unorm
        | wgpu::TextureFormat::Rg11b10Ufloat
        | wgpu::TextureFormat::Bgra8UnormSrgb
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Depth24PlusStencil8 => 4,
        wgpu::TextureFormat::R64Uint
        | wgpu::TextureFormat::Rg32Uint
        | wgpu::TextureFormat::Rg32Sint
        | wgpu::TextureFormat::Rg32Float
        | wgpu::TextureFormat::Rgba16Uint
        | wgpu::TextureFormat::Rgba16Sint
        | wgpu::TextureFormat::Rgba16Float
        | wgpu::TextureFormat::Rgba16Unorm
        | wgpu::TextureFormat::Rgba16Snorm
        | wgpu::TextureFormat::Depth32FloatStencil8 => 8,
        wgpu::TextureFormat::Rgba32Float
        | wgpu::TextureFormat::Rgba32Uint
        | wgpu::TextureFormat::Rgba32Sint => 16,
        _ => 4,
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Texture, TextureError);

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
