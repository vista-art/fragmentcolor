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

pub(crate) mod ktx2_loader;

mod platform;

use crate::{RenderContext, Size};
use image::{DynamicImage, GenericImageView};

mod error;
pub use error::*;

mod options;
pub use options::*;

mod read;
pub mod region;
mod write;

pub(crate) use read::{read_texture_object_async, read_texture_object_sync};
pub use region::TextureRegion;

pub mod meta;
pub use meta::{
    TextureClass, TextureDim, TextureScalarKind, TextureStorageAccess, TextureStorageFormat,
};

// Unified input type for creating textures (initial Rust subset)
#[derive(Debug, Clone, Default)]
pub enum TextureData {
    /// No initial pixel data — allocate the texture and leave the contents
    /// undefined. Required for `create_storage_texture` when seed data isn't
    /// provided. Pair with `options.size` (mandatory for this variant) and
    /// `options.format`.
    #[default]
    Empty,
    Bytes(Vec<u8>),
    Path(std::path::PathBuf),
    CloneOf(Texture),
    Url(String),
    DynamicImage(DynamicImage),
    /// In-memory KTX2 container bytes (decoded by the `ktx2` crate). Distinct
    /// from `Bytes`/`Path`/`Url` because the KTX2 path trusts the file's own
    /// declared format and skips the JPEG/PNG sRGB-inference + CPU mipmap
    /// chain. Use this for pre-baked compressed textures (BC7, ASTC, ETC2)
    /// from your asset pipeline.
    Ktx2Bytes(Vec<u8>),
    /// Path to a `.ktx2` file on disk.
    Ktx2Path(std::path::PathBuf),
    /// HTTP(S) URL pointing at a `.ktx2` file. Fetched the same way as `Url`.
    Ktx2Url(String),
    /// A pre-computed CPU mipmap chain produced by
    /// [`TextureMipChain::prepare`] (encoded image bytes) or
    /// [`TextureMipChain::prepare_raw`] (raw pixel bytes). Lets a worker
    /// thread (or any non-render thread) do the decode + mip generation,
    /// then hand the prepared chain to `Renderer::create_texture` for a
    /// GPU-only upload. The library already runs that prep on a background
    /// thread for `Bytes`/`Path`/`Url`/`DynamicImage` inputs; reach for this
    /// only when sharing a chain across textures or when you want explicit
    /// control over which thread does the work. Cross-language callers
    /// typically don't construct this variant — they hand `TextureMipChain` directly to
    /// constructing this variant directly. They hand a `TextureMipChain`
    /// straight to `Renderer::create_texture` and the unified entry point
    /// dispatches via `From<TextureMipChain> for TextureInput`.
    Prepared(TextureMipChain),
}

/// A pre-computed CPU mipmap chain. Build with [`TextureMipChain::prepare`]
/// (encoded image bytes) or [`TextureMipChain::prepare_raw`] (raw pixel bytes),
/// then pass to [`crate::Renderer::create_texture`] (the unified entry
/// point dispatches via `From<TextureMipChain> for TextureInput`).
///
/// Levels are tightly-packed bytes, level 0 first, with `bytes_per_pixel(format) *
/// max(1, base_w >> level) * max(1, base_h >> level)` bytes per level.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/core/texture_mip_chain/texture_mip_chain.md")]
pub struct TextureMipChain {
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) base_size: (u32, u32),
    /// Tightly-packed mip levels, level 0 first. Wrapped in `Arc` so cloning
    /// the handle (or its enclosing `TextureInput::Prepared` variant) does
    /// not duplicate the (potentially large) byte buffers.
    pub(crate) levels: std::sync::Arc<Vec<Vec<u8>>>,
}

crate::impl_fc_kind!(TextureMipChain, "TextureMipChain");
// Brand-anchored TryFrom<&JsValue> so the JS dispatch in TextureInput's
// TryFrom can detect a TextureMipChain handle (recovered via __wbg_ptr +
// __fc_kind brand) and route it as TextureInput::Prepared.
crate::impl_js_bridge!(TextureMipChain, crate::texture::TextureError);

impl TextureMipChain {
    /// Build a mip chain from a [`TextureInput`]. Pure CPU work — call from
    /// any thread (worker, thread pool, async task), then hand the chain to
    /// [`crate::Renderer::create_texture`] on the renderer thread for a
    /// GPU-only upload.
    ///
    /// Same `TextureInput` transport as `Renderer::create_texture` and
    /// `Renderer::create_storage_texture` — one vocabulary across the API.
    /// The `From<T>` impls cover the common call shapes:
    ///
    /// ```ignore
    /// // Encoded image bytes (PNG / JPEG / etc.) — size inferred from the image.
    /// TextureMipChain::prepare((png_bytes, TextureFormat::Rgba8UnormSrgb))?;
    ///
    /// // Raw pixel bytes — caller declares the dimensions.
    /// TextureMipChain::prepare((rgba_bytes, TextureFormat::Rgba8UnormSrgb, [w, h]))?;
    /// ```
    ///
    /// `prepare` requires a sync-friendly `data` variant: `Bytes`,
    /// `DynamicImage`, or `Path` (file IO). `Url` (needs async fetch),
    /// `Ktx2*` (already pre-baked), `Prepared` (already a chain),
    /// `CloneOf` (existing texture), and `Empty` (nothing to chain) all
    /// return [`TextureError::InvalidInput`].
    ///
    /// Supported formats for the mipmap chain: `Rgba8Unorm`,
    /// `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`,
    /// `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Other formats
    /// return [`TextureError::UnsupportedMipmapFormat`].
    #[lsp_doc("docs/api/core/texture_mip_chain/prepare.md")]
    pub fn prepare(input: impl Into<crate::TextureInput>) -> Result<TextureMipChain, TextureError> {
        let crate::TextureInput { data, options } = input.into();
        let public_format = options.format;
        let wfmt: wgpu::TextureFormat = public_format.into();
        if !format_supports_cpu_mipmaps(wfmt) {
            return Err(TextureError::UnsupportedMipmapFormat {
                format: public_format,
            });
        }
        let image = match data {
            // Encoded bytes path. If `options.size` is present, treat as raw
            // pixel data; otherwise decode internally.
            TextureData::Bytes(bytes) => match options.size {
                None => image::load_from_memory(&bytes)?,
                Some(size) => {
                    let extent: wgpu::Extent3d = size.into();
                    if extent.width == 0 || extent.height == 0 {
                        return Err(TextureError::InvalidInput(
                            "prepare: size must be non-zero in both dimensions".into(),
                        ));
                    }
                    let bpp = bytes_per_pixel(wfmt) as usize;
                    let expected = bpp * (extent.width as usize) * (extent.height as usize);
                    if bytes.len() < expected {
                        return Err(TextureError::InvalidInput(format!(
                            "prepare: expected {} bytes for {}x{} @ {} bpp, got {}",
                            expected,
                            extent.width,
                            extent.height,
                            bpp,
                            bytes.len()
                        )));
                    }
                    wrap_raw_bytes_as_dynamic_image(
                        &bytes[..expected],
                        extent.width,
                        extent.height,
                        wfmt,
                    )?
                }
            },
            TextureData::DynamicImage(image) => image,
            TextureData::Path(path) => image::open(path)?,
            // Async / pre-baked / non-prep variants don't fit the sync CPU
            // mipmap pipeline. Surface a clear typed error so callers can
            // match on the same `InvalidInput` variant they already match
            // on for shape mismatches.
            TextureData::Empty => {
                return Err(TextureError::InvalidInput(
                    "prepare: TextureData::Empty has no bytes to chain".into(),
                ));
            }
            TextureData::Url(_) => {
                return Err(TextureError::InvalidInput(
                    "prepare: TextureData::Url requires async fetch — fetch bytes first, then call prepare with the bytes".into(),
                ));
            }
            TextureData::Ktx2Bytes(_) | TextureData::Ktx2Path(_) | TextureData::Ktx2Url(_) => {
                return Err(TextureError::InvalidInput(
                    "prepare: KTX2 inputs already carry a pre-baked mip chain — pass them directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::Prepared(_) => {
                return Err(TextureError::InvalidInput(
                    "prepare: TextureData::Prepared is already a TextureMipChain — pass it directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::CloneOf(_) => {
                return Err(TextureError::InvalidInput(
                    "prepare: TextureData::CloneOf wraps an existing texture; nothing to chain"
                        .into(),
                ));
            }
        };
        Self::from_dynamic_image(&image, public_format, wfmt)
    }

    fn from_dynamic_image(
        image: &DynamicImage,
        public_format: crate::TextureFormat,
        format: wgpu::TextureFormat,
    ) -> Result<TextureMipChain, TextureError> {
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidInput(
                "prepare: decoded image has zero width or height".into(),
            ));
        }
        let levels = build_mip_chain_bytes(image, format).map_err(|err| match err {
            // Surface the format mismatch as the typed variant so callers
            // matching on UnsupportedMipmapFormat catch it both at the
            // top-level guard and in the inner dispatcher.
            TextureError::CreateTextureError(_) => TextureError::UnsupportedMipmapFormat {
                format: public_format,
            },
            other => other,
        })?;
        Ok(TextureMipChain {
            format,
            base_size: (width, height),
            levels: std::sync::Arc::new(levels),
        })
    }
}

impl TextureMipChain {
    /// The wgpu format the chain was prepared for.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Base level dimensions (level 0).
    pub fn base_size(&self) -> (u32, u32) {
        self.base_size
    }

    /// Tightly-packed bytes per mip level, level 0 first.
    pub fn levels(&self) -> &[Vec<u8>] {
        &self.levels
    }

    /// Number of mip levels in the chain (>= 1).
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

impl From<&[u8]> for TextureData {
    fn from(v: &[u8]) -> Self {
        TextureData::Bytes(v.to_vec())
    }
}
impl From<Vec<u8>> for TextureData {
    fn from(v: Vec<u8>) -> Self {
        TextureData::Bytes(v)
    }
}
impl From<&Vec<u8>> for TextureData {
    fn from(v: &Vec<u8>) -> Self {
        TextureData::Bytes(v.clone())
    }
}
impl From<&std::path::Path> for TextureData {
    fn from(p: &std::path::Path) -> Self {
        TextureData::Path(p.to_path_buf())
    }
}
impl From<std::path::PathBuf> for TextureData {
    fn from(p: std::path::PathBuf) -> Self {
        TextureData::Path(p)
    }
}
impl From<&std::path::PathBuf> for TextureData {
    fn from(p: &std::path::PathBuf) -> Self {
        TextureData::Path(p.clone())
    }
}
impl From<&Texture> for TextureData {
    fn from(t: &Texture) -> Self {
        TextureData::CloneOf(t.clone())
    }
}
impl From<TextureMipChain> for TextureData {
    fn from(chain: TextureMipChain) -> Self {
        TextureData::Prepared(chain)
    }
}

/// Uniffi-marshallable mirror of [`TextureInput`]. Drops the `DynamicImage`
/// variant (the `image` crate's `DynamicImage` isn't an FFI-friendly type;
/// callers on every non-Rust platform reach the same outcome by passing
/// `Bytes` and letting the library decode internally) and uses owned types
/// uniffi can lower across the FFI (`String` for paths, `Arc<Object>` for
/// handles).
///
/// The Rust core stays on `TextureInput`; this enum exists only so mobile
/// bindings can carry a concrete variant. Web and Python use their own
/// dispatch (`TryFrom<JsValue>` / Python kwargs) and never see this type.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum TextureInputMobile {
    Bytes(Vec<u8>),
    Path(String),
    Url(String),
    CloneOf(std::sync::Arc<Texture>),
    Prepared(std::sync::Arc<TextureMipChain>),
    Ktx2Bytes(Vec<u8>),
    Ktx2Path(String),
    Ktx2Url(String),
}

#[cfg(mobile)]
impl From<TextureInputMobile> for TextureData {
    fn from(input: TextureInputMobile) -> Self {
        match input {
            TextureInputMobile::Bytes(b) => TextureData::Bytes(b),
            TextureInputMobile::Path(s) => TextureData::Path(std::path::PathBuf::from(s)),
            TextureInputMobile::Url(s) => TextureData::Url(s),
            TextureInputMobile::CloneOf(t) => TextureData::CloneOf((*t).clone()),
            TextureInputMobile::Prepared(c) => TextureData::Prepared((*c).clone()),
            TextureInputMobile::Ktx2Bytes(b) => TextureData::Ktx2Bytes(b),
            TextureInputMobile::Ktx2Path(s) => TextureData::Ktx2Path(std::path::PathBuf::from(s)),
            TextureInputMobile::Ktx2Url(s) => TextureData::Ktx2Url(s),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureData {
    type Error = crate::texture::error::TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer, Uint8Array, Uint8ClampedArray};
        use wasm_bindgen::JsCast;

        // Case: TextureMipChain handle (built off-thread via TextureMipChain.prepare).
        // Detected through the branded __fc_kind + __wbg_ptr anchor so it
        // survives bundler renaming. Must come before the Uint8Array / object
        // cases because wasm-bindgen handles also satisfy `is_object()`.
        if let Ok(chain) = TextureMipChain::try_from(value) {
            return Ok(TextureData::Prepared(chain));
        }

        // Case: Uint8Array (fast path)
        if let Some(u8a) = value.dyn_ref::<Uint8Array>() {
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes));
        }

        // Case: Uint8ClampedArray (ImageData.data)
        if let Some(u8c) = value.dyn_ref::<Uint8ClampedArray>() {
            let mut bytes = vec![0u8; u8c.length() as usize];
            u8c.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes));
        }

        // Case: ArrayBuffer
        if let Some(buf) = value.dyn_ref::<ArrayBuffer>() {
            // Guard against detached buffers via byte_length() == 0; safe fallback for empty too.
            if buf.byte_length() == 0 {
                return Ok(TextureData::Bytes(Vec::new()));
            }
            let u8a = Uint8Array::new(buf);
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes));
        }

        // Case: ImageData -> use its backing data (Clamped<Vec<u8>> on wasm)
        if let Some(image_data) = value.dyn_ref::<web_sys::ImageData>() {
            let data = image_data.data();
            let bytes = data.0;
            return Ok(TextureData::Bytes(bytes));
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
            return Ok(TextureData::Bytes(bytes));
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
            return Ok(TextureData::Bytes(bytes));
        }

        // Case: HTMLImageElement -> pass through as URL
        if let Some(img) = value.dyn_ref::<web_sys::HtmlImageElement>() {
            return Ok(TextureData::Url(img.src()));
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
            return Ok(TextureData::Bytes(bytes));
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
            return Ok(TextureData::Url(url));
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

impl From<TextureId> for u64 {
    fn from(value: TextureId) -> Self {
        value.0
    }
}

// uniffi cannot derive `Record` on tuple structs (`pub struct TextureId(pub u64)`);
// register it as a custom type that lowers to `u64` across the FFI boundary.
// `From<u64> for TextureId` and `From<TextureId> for u64` provide the conversions.
#[cfg(mobile)]
uniffi::custom_type!(TextureId, u64);

#[cfg(wasm)]
crate::impl_js_bridge!(TextureId, crate::texture::TextureError);

#[cfg(wasm)]
pub(crate) fn js_to_texture_id(
    value: &wasm_bindgen::JsValue,
) -> Result<TextureId, crate::texture::TextureError> {
    if let Some(number) = value.as_f64() {
        if number.is_sign_negative() {
            return Err(crate::texture::TextureError::Error(
                "TextureId must be a non-negative number".into(),
            ));
        }
        return Ok(TextureId(number as u64));
    }
    TextureId::try_from(value)
}

#[cfg(wasm)]
pub(crate) fn js_to_texture_bytes(
    value: &wasm_bindgen::JsValue,
) -> Result<Vec<u8>, crate::texture::TextureError> {
    match TextureData::try_from(value)? {
        TextureData::Bytes(bytes) => Ok(bytes),
        _ => Err(crate::texture::TextureError::Error(
            "Expected raw byte data".into(),
        )),
    }
}

#[cfg(python)]
pub(crate) fn py_to_texture_id<'py>(
    any: &pyo3::Bound<'py, pyo3::PyAny>,
) -> pyo3::PyResult<TextureId> {
    if let Ok(number) = any.extract::<u64>() {
        return Ok(TextureId(number));
    }
    if let Ok(bound) = any.downcast::<TextureId>() {
        return Ok(*bound.borrow());
    }
    Err(crate::error::PyFragmentColorError::new_err(
        "Expected a TextureId or integer id",
    ))
}

#[cfg(python)]
pub(crate) fn py_to_texture_bytes<'py>(
    any: &pyo3::Bound<'py, pyo3::PyAny>,
) -> pyo3::PyResult<Vec<u8>> {
    if let Ok(bytes) = any.extract::<Vec<u8>>() {
        return Ok(bytes);
    }
    if let Ok(array) = any.downcast::<numpy::PyArrayDyn<u8>>() {
        use numpy::PyArrayMethods;

        let view = array.readonly();
        let bytes = view.as_slice().map_err(|_| {
            crate::error::PyFragmentColorError::new_err("ndarray must be contiguous")
        })?;
        return Ok(bytes.to_vec());
    }
    Err(crate::error::PyFragmentColorError::new_err(
        "Expected raw byte data",
    ))
}

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(mobile, derive(uniffi::Object))]
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
    #[lsp_doc("docs/api/core/texture/id.md")]
    pub fn id(&self) -> &TextureId {
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

    #[lsp_doc("docs/api/core/texture/write.md")]
    pub fn write(&self, data: &[u8]) -> Result<(), TextureError> {
        // Whole-texture write: TextureRegion::default() carries zeros that the
        // write path interprets as "infer the full extent".
        write::write(self, data, TextureRegion::default())
    }

    #[lsp_doc("docs/api/core/texture/write_region.md")]
    pub fn write_region(
        &self,
        data: &[u8],
        region: impl Into<TextureRegion>,
    ) -> Result<(), TextureError> {
        write::write(self, data, region.into())
    }

    #[lsp_doc("docs/api/core/texture/get_image.md")]
    pub fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        read::get_image(self)
    }

    #[lsp_doc("docs/api/core/texture/get_image_async.md")]
    pub async fn get_image_async(&self) -> Result<Vec<u8>, TextureError> {
        read::get_image_async(self).await
    }
}

// Metadata for textures parsed from shader source; users do not construct directly.
#[cfg_attr(python, pyo3::pyclass)]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureMeta {
    pub id: TextureId,
    pub dim: TextureDim,
    pub arrayed: bool,
    pub class: TextureClass,
    /// Whether the shader ever samples this texture (`textureSample*`). When false
    /// (only `textureLoad` / image-ops are used), the bind-group layout can request
    /// a non-filterable sample type, which unlocks formats like Rgba32Float as a
    /// sampled source without requiring the `FLOAT32_FILTERABLE` feature.
    pub sampled: bool,
}

impl TextureMeta {
    pub fn with_id_only(id: TextureId) -> Self {
        TextureMeta {
            id,
            dim: TextureDim::D2,
            arrayed: false,
            class: TextureClass::Sampled {
                kind: TextureScalarKind::Float,
                multi: false,
            },
            sampled: true,
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

    /// Creates a texture from a file.
    ///
    /// `format_override = None` infers the format from the image's color type.
    /// `generate_mipmaps = true` builds a full mip chain at upload (only for
    /// formats that share the source's RGBA8 byte layout — see `from_loaded_image`).
    pub(crate) fn from_file(
        context: &RenderContext,
        path: impl AsRef<Path>,
        format_override: Option<wgpu::TextureFormat>,
        generate_mipmaps: bool,
    ) -> Result<Self, TextureError> {
        let image = image::open(path)?;
        Ok(Self::from_loaded_image(
            context,
            &image,
            format_override,
            generate_mipmaps,
        ))
    }

    /// Creates a texture from encoded image bytes (PNG/JPEG/HDR, etc.).
    pub(crate) fn from_bytes(
        context: &RenderContext,
        bytes: &[u8],
        format_override: Option<wgpu::TextureFormat>,
        generate_mipmaps: bool,
    ) -> Result<Self, TextureError> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_loaded_image(
            context,
            &image,
            format_override,
            generate_mipmaps,
        ))
    }

    /// Creates a texture from raw pixel bytes with explicit size/format.
    /// The data layout is tightly packed with bytes_per_row = bpp * width.
    ///
    /// When `generate_mipmaps = true`, a full mip chain is built CPU-side via the
    /// dispatcher in [`write_raw_bytes_levels`]. Mipmaps run for every format
    /// `format_supports_cpu_mipmaps` accepts (Rgba8/Bgra8 family, R8, Rg8, R16,
    /// Rg16, Rgba16); other formats fall back to a single level.
    pub(crate) fn from_raw_bytes(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        data: &[u8],
        generate_mipmaps: bool,
    ) -> Result<Self, TextureError> {
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let bpp = bytes_per_pixel(format);

        let expected = (size.width as usize)
            .saturating_mul(size.height as usize)
            .saturating_mul(size.depth_or_array_layers as usize)
            .saturating_mul(bpp as usize);
        if data.len() < expected {
            return Err(TextureError::CreateTextureError(
                "Insufficient raw data for declared size/format".into(),
            ));
        }

        let want_mips = generate_mipmaps && format_supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(size.width, size.height)
        } else {
            1
        };

        let descriptor =
            Self::texture_descriptor("Raw Texture", size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_raw_bytes_levels(context, data, &texture, size, format, mip_level_count)?;

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

    /// Internal method to create a TextureObject from a DynamicImage instance.
    ///
    /// `format_override = None` infers the format from `image.color()`. The image is
    /// re-decoded to match the target format (e.g. `R16Unorm` triggers `to_luma16()`,
    /// preserving 16-bit precision). Mipmaps are generated CPU-side via
    /// `image::imageops::resize` for every format the dispatcher recognizes — see
    /// `format_supports_cpu_mipmaps`. Unrecognized formats fall back to writing
    /// `to_rgba8` bytes as a single level (best-effort, may misrender for non-RGBA8 layouts).
    pub(crate) fn from_loaded_image(
        context: &RenderContext,
        image: &DynamicImage,
        format_override: Option<wgpu::TextureFormat>,
        generate_mipmaps: bool,
    ) -> Self {
        let label = "Source Texture from Loaded Image";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = format_override.unwrap_or_else(|| infer_format_from_image(image.color()));

        let want_mips = generate_mipmaps && format_supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(width, height)
        } else {
            1
        };

        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let descriptor = Self::texture_descriptor(label, size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_image_levels(context, image, &texture, size, format, mip_level_count);

        let sampler = create_default_sampler(&context.device);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(SamplerOptions::default()),
            format,
            usage,
        }
    }

    /// Create a TextureObject from a pre-built CPU mip chain. Pure GPU writes —
    /// no decode, no resize. Used by the `TextureInput::Prepared` path.
    pub(crate) fn from_prepared_chain(
        context: &RenderContext,
        chain: TextureMipChain,
    ) -> Result<Self, TextureError> {
        let (w, h) = chain.base_size;
        if w == 0 || h == 0 {
            return Err(TextureError::CreateTextureError(
                "TextureMipChain base_size must be non-zero".into(),
            ));
        }
        if chain.levels.is_empty() {
            return Err(TextureError::CreateTextureError(
                "TextureMipChain must contain at least one level".into(),
            ));
        }
        let levels = chain.levels.as_slice();
        let format = chain.format;
        let bpp = bytes_per_pixel(format);
        if bpp == 0 {
            return Err(TextureError::CreateTextureError(format!(
                "TextureMipChain format {:?} has zero bytes-per-pixel",
                format
            )));
        }

        for (i, level_bytes) in levels.iter().enumerate() {
            let level_w = (w >> i).max(1);
            let level_h = (h >> i).max(1);
            let expected = (bpp as usize) * (level_w as usize) * (level_h as usize);
            if level_bytes.len() != expected {
                return Err(TextureError::CreateTextureError(format!(
                    "TextureMipChain level {} has {} bytes, expected {} ({}x{} @ {} bpp)",
                    i,
                    level_bytes.len(),
                    expected,
                    level_w,
                    level_h,
                    bpp
                )));
            }
        }

        let size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let mip_level_count = levels.len() as u32;
        let descriptor =
            Self::texture_descriptor("Prepared Texture", size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        for (i, level_bytes) in levels.iter().enumerate() {
            let level_w = (w >> i).max(1);
            let level_h = (h >> i).max(1);
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: i as u32,
                    origin: wgpu::Origin3d::ZERO,
                },
                level_bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpp * level_w),
                    rows_per_image: Some(level_h),
                },
                wgpu::Extent3d {
                    width: level_w,
                    height: level_h,
                    depth_or_array_layers: 1,
                },
            );
        }

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
}

/// Number of mip levels in a full chain that goes down to 1×1 for the given dimensions.
/// Returns 1 for any zero or 1×1 input.
pub(crate) fn mip_level_count_for(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim <= 1 {
        return 1;
    }
    1 + max_dim.ilog2()
}

/// True for formats the CPU mipmap dispatcher in [`write_image_levels`] can fill.
/// Covers every wgpu format we know how to decode a `DynamicImage` into via the
/// `image` crate's `to_*` helpers.
pub(crate) fn format_supports_cpu_mipmaps(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
            | wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::Rg8Unorm
            | wgpu::TextureFormat::R16Unorm
            | wgpu::TextureFormat::Rg16Unorm
            | wgpu::TextureFormat::Rgba16Unorm
    )
}

/// Map an `image::ColorType` to a sensible default `wgpu::TextureFormat` for sampling.
pub(crate) fn infer_format_from_image(color: image::ColorType) -> wgpu::TextureFormat {
    match color {
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
    }
}

/// Decode `image` into the right pixel buffer for `format`, then upload it as
/// mip 0 — and (when `mip_level_count > 1`) iteratively downsample with the
/// Triangle filter for each subsequent level. This is the dispatch that lets us
/// preserve 16-bit precision (`R16Unorm` from a 16-bit grayscale PNG) instead of
/// silently going through `to_rgba8`.
///
/// Each branch picks the matching `image` crate decoder so the channel layout and
/// bytes-per-pixel agree with the wgpu format. Unrecognized formats fall back to
/// writing `to_rgba8` bytes as a single level — best-effort, may misrender for
/// non-RGBA8 layouts (caller should have caught this via `format_supports_cpu_mipmaps`
/// when picking `mip_level_count`).
pub(crate) fn write_image_levels(
    context: &RenderContext,
    image: &DynamicImage,
    texture: &wgpu::Texture,
    full_size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) {
    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            write_pixel_levels(
                context,
                image.to_rgba8(),
                texture,
                full_size,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::R8Unorm => {
            write_pixel_levels(
                context,
                image.to_luma8(),
                texture,
                full_size,
                mip_level_count,
                1,
            );
        }
        wgpu::TextureFormat::Rg8Unorm => {
            write_pixel_levels(
                context,
                image.to_luma_alpha8(),
                texture,
                full_size,
                mip_level_count,
                2,
            );
        }
        wgpu::TextureFormat::R16Unorm => {
            write_pixel_levels(
                context,
                image.to_luma16(),
                texture,
                full_size,
                mip_level_count,
                2,
            );
        }
        wgpu::TextureFormat::Rg16Unorm => {
            write_pixel_levels(
                context,
                image.to_luma_alpha16(),
                texture,
                full_size,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            write_pixel_levels(
                context,
                image.to_rgba16(),
                texture,
                full_size,
                mip_level_count,
                8,
            );
        }
        _ => {
            log::warn!(
                "from_loaded_image: unsupported texture format {:?}; writing to_rgba8 bytes as a single level (channels and bytes-per-pixel may not match the texture)",
                format
            );
            let source = image.to_rgba8();
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                source.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * full_size.width),
                    rows_per_image: Some(full_size.height),
                },
                full_size,
            );
        }
    }
}

/// Wrap raw pixel bytes (already laid out for `format` at `width x height`)
/// into a `DynamicImage` so [`build_mip_chain_bytes`] can downsample them with
/// the same Triangle filter used by the inline path. Mirrors the format
/// dispatch in [`write_raw_bytes_levels`] but produces an in-memory image
/// instead of writing to the GPU.
pub(crate) fn wrap_raw_bytes_as_dynamic_image(
    bytes: &[u8],
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> Result<DynamicImage, TextureError> {
    let pixel_count = (width as usize) * (height as usize);
    let wrap_err = |kind: &str| {
        TextureError::CreateTextureError(format!(
            "Failed to wrap raw bytes as {} for prepare_raw",
            kind
        ))
    };
    Ok(match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(width, height, bytes[..pixel_count * 4].to_vec())
                .ok_or_else(|| wrap_err("RgbaImage"))?,
        ),
        wgpu::TextureFormat::R8Unorm => DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(width, height, bytes[..pixel_count].to_vec())
                .ok_or_else(|| wrap_err("GrayImage"))?,
        ),
        wgpu::TextureFormat::Rg8Unorm => DynamicImage::ImageLumaA8(
            image::GrayAlphaImage::from_raw(width, height, bytes[..pixel_count * 2].to_vec())
                .ok_or_else(|| wrap_err("GrayAlphaImage"))?,
        ),
        wgpu::TextureFormat::R16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 2]);
            DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("Luma16"))?,
            )
        }
        wgpu::TextureFormat::Rg16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 4]);
            DynamicImage::ImageLumaA16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("LumaA16"))?,
            )
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 8]);
            DynamicImage::ImageRgba16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("Rgba16"))?,
            )
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "wrap_raw_bytes_as_dynamic_image: format {:?} is not supported",
                format
            )));
        }
    })
}

/// Pure-CPU mip chain builder used by [`TextureMipChain::prepare`]. Mirrors the format
/// dispatch in [`write_image_levels`] but buffers each level's bytes into a
/// `Vec<u8>` instead of writing to a wgpu texture. Caller must have validated
/// that `format` is in [`format_supports_cpu_mipmaps`].
pub(crate) fn build_mip_chain_bytes(
    image: &DynamicImage,
    format: wgpu::TextureFormat,
) -> Result<Vec<Vec<u8>>, TextureError> {
    let (w, h) = image.dimensions();
    let level_count = mip_level_count_for(w, h) as usize;
    let chain = match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            downsample_levels_to_bytes(image.to_rgba8(), level_count)
        }
        wgpu::TextureFormat::R8Unorm => downsample_levels_to_bytes(image.to_luma8(), level_count),
        wgpu::TextureFormat::Rg8Unorm => {
            downsample_levels_to_bytes(image.to_luma_alpha8(), level_count)
        }
        wgpu::TextureFormat::R16Unorm => downsample_levels_to_bytes(image.to_luma16(), level_count),
        wgpu::TextureFormat::Rg16Unorm => {
            downsample_levels_to_bytes(image.to_luma_alpha16(), level_count)
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            downsample_levels_to_bytes(image.to_rgba16(), level_count)
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "build_mip_chain_bytes: format {:?} is not supported",
                format
            )));
        }
    };
    Ok(chain)
}

/// Generic level-buffering: starting from `base`, push the bytes of each level
/// into a fresh `Vec<u8>`, then resample to half size with the Triangle filter.
/// Identical resampling to [`write_pixel_levels`] — same Triangle filter, same
/// floor-to-1 dimension shrink — so chains built here render identically to
/// the inline path.
fn downsample_levels_to_bytes<P>(
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    level_count: usize,
) -> Vec<Vec<u8>>
where
    P: image::Pixel + 'static,
    P::Subpixel: bytemuck::NoUninit + 'static,
{
    let mut levels = Vec::with_capacity(level_count);
    let mut current = base;
    for level in 0..level_count {
        let bytes: &[u8] = bytemuck::cast_slice(current.as_raw());
        levels.push(bytes.to_vec());
        if level + 1 < level_count {
            let next_w = (current.width() / 2).max(1);
            let next_h = (current.height() / 2).max(1);
            current = image::imageops::resize(
                &current,
                next_w,
                next_h,
                image::imageops::FilterType::Triangle,
            );
        }
    }
    levels
}

/// Wrap pre-decoded raw bytes in the right `ImageBuffer` for `format`, then upload
/// (and downsample for higher mip levels). Mirrors [`write_image_levels`] but starts
/// from a `&[u8]` buffer the caller has already laid out tightly. For 16-bit formats,
/// the byte slice is converted to `Vec<u16>` via `from_le_bytes` (alignment-safe and
/// matches WebGPU's little-endian element ordering).
pub(crate) fn write_raw_bytes_levels(
    context: &RenderContext,
    bytes: &[u8],
    texture: &wgpu::Texture,
    full_size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> Result<(), TextureError> {
    if mip_level_count == 1 {
        let bpp = bytes_per_pixel(format);
        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpp * full_size.width),
                rows_per_image: Some(full_size.height),
            },
            full_size,
        );
        return Ok(());
    }

    // Multi-level: wrap into the right ImageBuffer so write_pixel_levels can resample.
    let w = full_size.width;
    let h = full_size.height;
    let pixel_count = (w as usize) * (h as usize);

    let wrap_err = |kind: &str| {
        TextureError::CreateTextureError(format!(
            "Failed to wrap raw bytes as {} for mipmap generation",
            kind
        ))
    };

    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            let base = image::RgbaImage::from_raw(w, h, bytes[..pixel_count * 4].to_vec())
                .ok_or_else(|| wrap_err("RgbaImage"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 4);
        }
        wgpu::TextureFormat::R8Unorm => {
            let base: image::GrayImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count].to_vec())
                    .ok_or_else(|| wrap_err("GrayImage"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 1);
        }
        wgpu::TextureFormat::Rg8Unorm => {
            let base: image::GrayAlphaImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count * 2].to_vec())
                    .ok_or_else(|| wrap_err("GrayAlphaImage"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 2);
        }
        wgpu::TextureFormat::R16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 2]);
            let base: image::ImageBuffer<image::Luma<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Luma16"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 2);
        }
        wgpu::TextureFormat::Rg16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 4]);
            let base: image::ImageBuffer<image::LumaA<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("LumaA16"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 4);
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 8]);
            let base: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Rgba16"))?;
            write_pixel_levels(context, base, texture, full_size, mip_level_count, 8);
        }
        _ => {
            // Caller should have caught this via format_supports_cpu_mipmaps;
            // be defensive and write a single level.
            let bpp = bytes_per_pixel(format);
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpp * full_size.width),
                    rows_per_image: Some(full_size.height),
                },
                full_size,
            );
        }
    }
    Ok(())
}

/// Convert a tightly-packed byte slice into `Vec<u16>` by reading each pair of
/// bytes as a little-endian u16. Alignment-safe (the source `&[u8]` need not be
/// u16-aligned) and matches WebGPU's element ordering for 16-bit formats.
fn bytes_to_u16_vec(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/// Generic worker: upload `base` as mip 0, then iteratively downsample with the
/// Triangle filter until the chain is full. Works for any pixel type the `image`
/// crate's `imageops::resize` supports (Rgba8, Luma8, LumaA8, Luma16, LumaA16,
/// Rgba16). Pixel data is reinterpreted to bytes via `bytemuck::cast_slice`,
/// which is a no-op for `u8` subpixels and a little-endian byte view for `u16`.
/// All FragmentColor target platforms are little-endian, which matches WebGPU's
/// expected byte order for `*Unorm` formats.
pub(crate) fn write_pixel_levels<P>(
    context: &RenderContext,
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    texture: &wgpu::Texture,
    full_size: wgpu::Extent3d,
    mip_level_count: u32,
    bpp: u32,
) where
    P: image::Pixel + 'static,
    P::Subpixel: bytemuck::NoUninit + 'static,
{
    let mut current = base;
    let mut current_w = full_size.width;
    let mut current_h = full_size.height;

    for level in 0..mip_level_count {
        let bytes: &[u8] = bytemuck::cast_slice(current.as_raw());
        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpp * current_w),
                rows_per_image: Some(current_h),
            },
            wgpu::Extent3d {
                width: current_w,
                height: current_h,
                depth_or_array_layers: 1,
            },
        );

        if level + 1 < mip_level_count {
            let next_w = (current_w / 2).max(1);
            let next_h = (current_h / 2).max(1);
            current = image::imageops::resize(
                &current,
                next_w,
                next_h,
                image::imageops::FilterType::Triangle,
            );
            current_w = next_w;
            current_h = next_h;
        }
    }
}

pub(crate) fn bytes_per_pixel(format: wgpu::TextureFormat) -> u32 {
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
        let ti1: TextureData = (&bytes).into();
        match ti1 {
            TextureData::Bytes(b) => assert_eq!(b, bytes),
            _ => panic!("expected Bytes"),
        }

        let p = std::path::PathBuf::from("/tmp/img.png");
        let ti2: TextureData = (&p).into();
        match ti2 {
            TextureData::Path(pb) => assert_eq!(pb, p),
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
                .create_texture(TextureData::DynamicImage(img))
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

    // Story: mip_level_count_for matches the standard "down to 1×1" chain length.
    #[test]
    fn mip_level_count_for_known_dims() {
        assert_eq!(mip_level_count_for(0, 0), 1);
        assert_eq!(mip_level_count_for(1, 1), 1);
        assert_eq!(mip_level_count_for(2, 1), 2);
        assert_eq!(mip_level_count_for(1, 2), 2);
        assert_eq!(mip_level_count_for(4, 4), 3); // 4, 2, 1
        assert_eq!(mip_level_count_for(1024, 512), 11); // 1024 → 1
        // Non-power-of-two: chain length follows the largest dim.
        assert_eq!(mip_level_count_for(1000, 600), 10); // floor(log2(1000)) + 1
    }

    // Story: Default options request a mipmap chain; explicit opt-out yields a single level.
    #[test]
    fn dynamic_image_creates_mipmap_chain_by_default() {
        pollster::block_on(async move {
            // 8x8 solid color image — generates 4 mip levels (8, 4, 2, 1).
            let pixels = vec![200u8; 8 * 8 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::DynamicImage(img.clone()))
                .await
                .expect("create texture with mipmaps");
            assert_eq!(tex.object.inner.mip_level_count(), 4);

            // Opt-out: only mip 0
            let opts = TextureOptions {
                mipmaps: false,
                ..Default::default()
            };
            let tex_no_mip = r
                .create_texture((TextureData::DynamicImage(img), opts))
                .await
                .expect("create texture without mipmaps");
            assert_eq!(tex_no_mip.object.inner.mip_level_count(), 1);
        });
    }

    // Story: Format override is honored on the DynamicImage arm — same RGBA bytes,
    // reinterpreted as linear instead of sRGB. Mipmap chain is still generated.
    #[test]
    fn format_override_honored_for_image_input() {
        pollster::block_on(async move {
            let pixels = vec![128u8; 4 * 4 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(4, 4, pixels).unwrap());
            let r = crate::Renderer::new();

            let opts = TextureOptions {
                format: crate::TextureFormat::Rgba8Unorm,
                ..Default::default()
            };
            let tex = r
                .create_texture((TextureData::DynamicImage(img), opts))
                .await
                .expect("create texture with explicit linear format");
            assert_eq!(tex.object.format, wgpu::TextureFormat::Rgba8Unorm);
            assert_eq!(tex.object.inner.mip_level_count(), 3); // 4, 2, 1
        });
    }

    // Story: 16-bit grayscale source (e.g. height-map PNG) is decoded with
    // to_luma16 — preserving precision — and uploaded as R16Unorm with mips.
    #[test]
    fn luma16_image_uploads_as_r16unorm_with_mipmaps() {
        pollster::block_on(async move {
            // 4×4 ramp where each pixel needs the full u16 range to round-trip.
            let pixels: Vec<u16> = (0..16).map(|i| i * 4096).collect();
            let buf = image::ImageBuffer::<image::Luma<u16>, _>::from_vec(4, 4, pixels).unwrap();
            let img = image::DynamicImage::ImageLuma16(buf);

            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::DynamicImage(img))
                .await
                .expect("create R16Unorm texture from Luma16 image");
            assert_eq!(tex.object.format, wgpu::TextureFormat::R16Unorm);
            assert_eq!(tex.object.inner.mip_level_count(), 3); // 4, 2, 1
        });
    }

    // Story: 8-bit grayscale source goes to R8Unorm with the matching CPU mipmap path.
    #[test]
    fn luma8_image_uploads_as_r8unorm_with_mipmaps() {
        pollster::block_on(async move {
            let pixels: Vec<u8> = (0..16u8).map(|i| i * 16).collect();
            let buf = image::GrayImage::from_vec(4, 4, pixels).unwrap();
            let img = image::DynamicImage::ImageLuma8(buf);

            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::DynamicImage(img))
                .await
                .expect("create R8Unorm texture from Luma8 image");
            assert_eq!(tex.object.format, wgpu::TextureFormat::R8Unorm);
            assert_eq!(tex.object.inner.mip_level_count(), 3); // 4, 2, 1
        });
    }

    // Story: prepare with the From<(bytes, format)> impl produces a chain
    // whose level count and per-level byte sizes match what Renderer::create_texture
    // computes inline. Mirrors the KTX2 "trust the caller's prebaked levels"
    // path but for runtime-prepared chains coming off a worker thread.
    #[test]
    fn prepare_builds_mipmap_chain_matching_inline_path() {
        // 8×8 RGBA8 PNG → 4 mip levels (8, 4, 2, 1).
        let pixels = vec![123u8; 8 * 8 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        // From<(&[u8], TextureFormat)> — encoded image path.
        let chain =
            TextureMipChain::prepare((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
                .expect("prepare ok");
        assert_eq!(chain.level_count(), 4);
        assert_eq!(chain.base_size(), (8, 8));
        assert_eq!(chain.format(), wgpu::TextureFormat::Rgba8UnormSrgb);
        let expected = [4 * 8 * 8, 4 * 4 * 4, 4 * 2 * 2, 4 * 1 * 1];
        for (i, level) in chain.levels().iter().enumerate() {
            assert_eq!(level.len(), expected[i], "level {i} byte count");
        }
    }

    // Story: the raw-pixel path is selected by providing a `size` — same
    // single `prepare` entry, the From<(bytes, format, size)> impl carries
    // it. Exercises tuple, array, and Size as the size argument.
    #[test]
    fn prepare_raw_builds_chain_from_raw_pixels() {
        let pixels = vec![200u8; 8 * 8 * 4];
        // From<(&[u8], TextureFormat, (u32, u32))>
        let chain = TextureMipChain::prepare((
            pixels.as_slice(),
            crate::TextureFormat::Rgba8UnormSrgb,
            (8u32, 8u32),
        ))
        .expect("prepare with tuple size");
        assert_eq!(chain.level_count(), 4);
        assert_eq!(chain.base_size(), (8, 8));

        // From<(&[u8], TextureFormat, [u32; 2])>
        let chain2 = TextureMipChain::prepare((
            pixels.as_slice(),
            crate::TextureFormat::Rgba8UnormSrgb,
            [8u32, 8u32],
        ))
        .expect("prepare with array size");
        assert_eq!(chain2.base_size(), (8, 8));

        // Explicit `TextureInput` struct literal — same transport as
        // `Renderer::create_texture` and `create_storage_texture`.
        let chain3 = TextureMipChain::prepare(crate::TextureInput {
            data: crate::TextureData::Bytes(pixels.clone()),
            options: crate::TextureOptions {
                size: Some(crate::Size::from([8u32, 8u32])),
                format: crate::TextureFormat::Rgba8UnormSrgb,
                ..Default::default()
            },
        })
        .expect("prepare with explicit TextureInput");
        assert_eq!(chain3.base_size(), (8, 8));
    }

    // Story: prepare's three failure modes (decode, format, shape) surface as
    // distinct TextureError variants so callers logging tile-cache errors can
    // distinguish "this tile's bytes are corrupt" from "this format isn't
    // supported by FC's CPU mipmap path" from "the bytes don't match the
    // declared shape" without parsing strings.
    #[test]
    fn prepare_error_variants_are_distinct() {
        // 1) Decode failure → MalformedImageError (from `image::ImageError`).
        let garbage = vec![0xFFu8; 32];
        let err =
            TextureMipChain::prepare((garbage.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
                .expect_err("garbage bytes should fail to decode");
        assert!(
            matches!(err, TextureError::MalformedImageError(_)),
            "expected MalformedImageError, got {err:?}"
        );

        // 2) Unsupported format → UnsupportedMipmapFormat carrying the
        //    public TextureFormat the caller passed in.
        let pixels = vec![0u8; 4 * 4 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(4, 4, pixels).unwrap());
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let err = TextureMipChain::prepare((buf.as_slice(), crate::TextureFormat::Rgba32Float))
            .expect_err("Rgba32Float not supported by CPU mipmap path");
        match err {
            TextureError::UnsupportedMipmapFormat { format } => {
                assert_eq!(format, crate::TextureFormat::Rgba32Float);
            }
            other => panic!("expected UnsupportedMipmapFormat, got {other:?}"),
        }

        // 3) Shape mismatch (raw path) → InvalidInput.
        let too_few_bytes = vec![0u8; 16];
        let err = TextureMipChain::prepare((
            too_few_bytes.as_slice(),
            crate::TextureFormat::Rgba8UnormSrgb,
            (32u32, 32u32),
        ))
        .expect_err("declared 32x32 RGBA but only gave 16 bytes");
        assert!(
            matches!(err, TextureError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );

        // 4) Zero size (raw path) → InvalidInput.
        let err = TextureMipChain::prepare((
            &[][..],
            crate::TextureFormat::Rgba8UnormSrgb,
            (0u32, 16u32),
        ))
        .expect_err("zero-width raw input should fail");
        assert!(
            matches!(err, TextureError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );
    }

    // Story: A prepared chain feeds back into create_texture and produces a
    // texture with the expected format + mip chain — the round-trip RemixBrush
    // uses for off-main-thread mip generation. Exercises both the
    // TextureInput::Prepared variant (Rust ergonomics) and the dedicated
    // create_texture_prepared cross-language entry point.
    #[test]
    fn prepared_input_round_trips_through_create_texture() {
        pollster::block_on(async move {
            let pixels = vec![77u8; 8 * 8 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .unwrap();
            let chain =
                TextureMipChain::prepare((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
                    .expect("prepare ok");
            let level_count = chain.level_count() as u32;
            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::Prepared(chain.clone()))
                .await
                .expect("create_texture from Prepared chain");
            assert_eq!(tex.object.format, wgpu::TextureFormat::Rgba8UnormSrgb);
            assert_eq!(tex.object.inner.mip_level_count(), level_count);
            let sz = tex.size();
            assert_eq!(sz.width, 8);
            assert_eq!(sz.height, 8);

            // create_texture(chain) reaches the same path via
            // From<TextureMipChain> for TextureInput. No second method needed.
            let tex2 = r
                .create_texture(chain)
                .await
                .expect("create_texture from chain");
            assert_eq!(tex2.object.format, wgpu::TextureFormat::Rgba8UnormSrgb);
            assert_eq!(tex2.object.inner.mip_level_count(), level_count);
        });
    }

    // Story: Raw bytes with R16Unorm format — caller hands us packed little-endian
    // u16s, we wrap them as a Luma16 buffer and resample for each mip level.
    #[test]
    fn raw_bytes_r16unorm_generates_mipmap_chain() {
        pollster::block_on(async move {
            // 4×4 image with each u16 cycling through the full range.
            let words: Vec<u16> = (0..16).map(|i| i * 4096).collect();
            let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
            for w in &words {
                bytes.extend_from_slice(&w.to_le_bytes());
            }
            let opts = TextureOptions {
                size: Some(crate::Size::from([4u32, 4u32])),
                format: crate::TextureFormat::R16Unorm,
                ..Default::default()
            };
            let r = crate::Renderer::new();
            let tex = r
                .create_texture((&bytes[..], opts))
                .await
                .expect("create R16Unorm texture from raw bytes");
            assert_eq!(tex.object.format, wgpu::TextureFormat::R16Unorm);
            assert_eq!(tex.object.inner.mip_level_count(), 3); // 4, 2, 1
        });
    }
}
