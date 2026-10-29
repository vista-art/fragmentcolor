use crate::shader::uniform::UniformData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use lsp_doc::lsp_doc;
use parking_lot::RwLock;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod sampler;
pub use sampler::*;

pub mod format;
pub use format::*;

pub mod usage;
pub use usage::TextureUsage;

pub(crate) mod ktx2_loader;

mod platform;
#[cfg(python)]
pub(crate) use platform::{
    PySamplerOptions, PyTextureId, PyTextureSource, PyTextureUsage, ndarray_input,
};
#[cfg(wasm)]
pub(crate) use platform::{read_format, read_texture_input, read_usage};

use crate::{RenderContext, Size};
use image::{DynamicImage, GenericImageView};

mod error;
pub use error::*;

mod options;
pub use options::*;

pub(crate) mod read;
pub mod region;
mod registration;
mod resize;
mod source;
mod write;

#[cfg(python)]
pub(crate) use read::read_pixels;
pub(crate) use read::{copy_pixels, read_pixels_from};
pub use region::TextureRegion;
#[cfg(mobile)]
pub use region::TextureRegionMobile;
pub(crate) use registration::Registration;
pub(crate) use resize::{check_extent, check_plane, check_size, resize};
pub(crate) use source::{SourceData, TextureSource};
pub(crate) use usage::check_usage;

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
    /// In-memory KTX2 container bytes (decoded by the `ktx2` crate). The KTX2
    /// path trusts the file's own declared format and skips the JPEG/PNG
    /// sRGB-inference + CPU mipmap chain. Use this for pre-baked compressed
    /// textures (BC7, ASTC, ETC2) from your asset pipeline. `Bytes`, `Path`
    /// and `Url` take the same path when the data starts with the KTX2 identifier.
    Ktx2Bytes(Vec<u8>),
    /// Path to a `.ktx2` file on disk.
    Ktx2Path(std::path::PathBuf),
    /// HTTP(S) URL pointing at a `.ktx2` file. Fetched the same way as `Url`.
    Ktx2Url(String),
    /// A pre-computed CPU mipmap chain produced by [`Mipmap::build`]. Lets a
    /// worker thread (or any non-render thread) do the decode + mip
    /// generation, then hand the prepared chain to `Renderer::create_texture`
    /// for a GPU-only upload. The library already runs that prep on a
    /// background thread for `Bytes`/`Path`/`Url` inputs; reach
    /// for this only when sharing a chain across textures or when you want
    /// explicit control over which thread does the work. Cross-language
    /// callers typically don't construct this variant directly — they hand a
    /// [`Mipmap`] straight to `Renderer::create_texture` and the unified
    /// entry point dispatches via `From<Mipmap> for TextureInput`.
    Prepared(Mipmap),
}

impl TextureData {
    /// Whether the data brings its own size, as every form but raw pixel bytes does.
    /// `Empty` and `CloneOf` hold no data to size.
    fn is_sized(&self) -> bool {
        match self {
            TextureData::Bytes(bytes) => ktx2_loader::is_ktx2(bytes),
            TextureData::Path(_)
            | TextureData::Url(_)
            | TextureData::Ktx2Bytes(_)
            | TextureData::Ktx2Path(_)
            | TextureData::Ktx2Url(_)
            | TextureData::Prepared(_) => true,
            TextureData::Empty | TextureData::CloneOf(_) => false,
        }
    }
}

/// A pre-computed CPU mipmap chain. Build with [`Mipmap::build`], then pass
/// to [`crate::Renderer::create_texture`] (the unified entry point
/// dispatches via `From<Mipmap> for TextureInput`).
///
/// Levels are tightly-packed bytes, level 0 first, with the format's bytes per pixel
/// times `max(1, base_w >> level) * max(1, base_h >> level)` bytes per level.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/texture/mipmap/mipmap.md")]
pub struct Mipmap {
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) base_size: (u32, u32),
    /// Tightly-packed mip levels, level 0 first. Wrapped in `Arc` so cloning
    /// the handle (or its enclosing `TextureInput::Prepared` variant) does
    /// not duplicate the (potentially large) byte buffers.
    pub(crate) levels: std::sync::Arc<Vec<Vec<u8>>>,
}

crate::impl_fc_kind!(Mipmap, "Mipmap");
// Brand-anchored TryFrom<&JsValue> so the JS dispatch in TextureInput's
// TryFrom can detect a Mipmap handle (recovered via __wbg_ptr +
// __fc_kind brand) and route it as TextureInput::Prepared.
crate::impl_js_bridge!(Mipmap, crate::texture::TextureError);

/// The side limit a chain decodes with. It has no device yet,
/// and `create_texture` checks its size against the device when it uploads.
const ANY_SIDE: u32 = u32::MAX;

impl Mipmap {
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
    /// Mipmap::build((png_bytes, TextureFormat::Rgba8UnormSrgb))?;
    ///
    /// // Raw pixel bytes — caller declares the dimensions.
    /// Mipmap::build((rgba_bytes, TextureFormat::Rgba8UnormSrgb, [w, h]))?;
    /// ```
    ///
    /// `build` requires a sync-friendly `data` variant: `Bytes` or
    /// `Path` (file IO). `Url` (needs async fetch),
    /// `Ktx2*` (already pre-baked), `Prepared` (already a chain),
    /// `CloneOf` (existing texture), and `Empty` (nothing to chain) all
    /// return [`TextureError::InvalidInput`].
    ///
    /// Supported formats for the mipmap chain: `Rgba8Unorm`,
    /// `Rgba8UnormSrgb`, `Bgra8Unorm`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`,
    /// `Rg16Unorm`, `Rgba16Unorm`, `Rgba16Float`, `Rgba32Float`, and the
    /// aliases for them. Other formats return [`TextureError::UnsupportedMipmapFormat`].
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build(input: impl Into<crate::TextureInput>) -> Result<Mipmap, TextureError> {
        let crate::TextureInput { data, options } = input.into();
        check_source(data.is_sized(), options.size)?;
        let public_format = options.format;
        let wfmt: wgpu::TextureFormat = public_format.into();
        if !supports_cpu_mipmaps(wfmt) {
            return Err(TextureError::UnsupportedMipmapFormat {
                format: public_format,
            });
        }
        // Raw pixels keep their bytes as level 0, and only the levels below it are resampled.
        let (image, raw) = match data {
            // Encoded bytes path. If `options.size` is present, treat as raw
            // pixel data; otherwise decode internally.
            TextureData::Bytes(bytes) => match options.size {
                None => (decode(&bytes, ANY_SIDE)?, None),
                Some(size) => {
                    let extent: wgpu::Extent3d = size.into();
                    if extent.width == 0 || extent.height == 0 {
                        return Err(TextureError::InvalidInput(
                            "build: size must be non-zero in both dimensions".into(),
                        ));
                    }
                    if extent.depth_or_array_layers != 1 {
                        return Err(TextureError::InvalidInput(format!(
                            "build: a mip chain resamples one layer, got a depth of {}",
                            extent.depth_or_array_layers
                        )));
                    }
                    let bpp = texel_size(wfmt)? as usize;
                    let expected = bpp * (extent.width as usize) * (extent.height as usize);
                    if bytes.len() != expected {
                        return Err(TextureError::InvalidInput(format!(
                            "build: expected {} bytes for {}x{} @ {} bpp, got {}",
                            expected,
                            extent.width,
                            extent.height,
                            bpp,
                            bytes.len()
                        )));
                    }
                    let image = bytes_as_image(&bytes, extent.width, extent.height, wfmt)?;
                    (image, Some(bytes))
                }
            },
            TextureData::Path(path) => (decode(&read_file(&path)?, ANY_SIDE)?, None),
            // Async / pre-baked / non-prep variants don't fit the sync CPU
            // mipmap pipeline. Surface a clear typed error so callers can
            // match on the same `InvalidInput` variant they already match
            // on for shape mismatches.
            TextureData::Empty => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Empty has no bytes to chain".into(),
                ));
            }
            TextureData::Url(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Url requires async fetch — fetch bytes first, then call build with the bytes".into(),
                ));
            }
            TextureData::Ktx2Bytes(_) | TextureData::Ktx2Path(_) | TextureData::Ktx2Url(_) => {
                return Err(TextureError::InvalidInput(
                    "build: KTX2 inputs already carry a pre-baked mip chain — pass them directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::Prepared(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Prepared is already a Mipmap — pass it directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::CloneOf(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::CloneOf wraps an existing texture; nothing to chain"
                        .into(),
                ));
            }
        };
        Self::from_dynamic_image(&image, public_format, wfmt, raw)
    }

    /// The chain of `image`, with `raw`, the caller's own bytes for it, as level 0.
    fn from_dynamic_image(
        image: &DynamicImage,
        public_format: crate::TextureFormat,
        format: wgpu::TextureFormat,
        raw: Option<Vec<u8>>,
    ) -> Result<Mipmap, TextureError> {
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidInput(
                "build: decoded image has zero width or height".into(),
            ));
        }
        let mut levels = build_mip_chain(image, format).map_err(|err| match err {
            // Surface the format mismatch as the typed variant so callers
            // matching on UnsupportedMipmapFormat catch it both at the
            // top-level guard and in the inner dispatcher.
            TextureError::CreateTextureError(_) => TextureError::UnsupportedMipmapFormat {
                format: public_format,
            },
            other => other,
        })?;
        if let (Some(raw), Some(top)) = (raw, levels.first_mut()) {
            *top = raw;
        }
        Ok(Mipmap {
            format,
            base_size: (width, height),
            levels: std::sync::Arc::new(levels),
        })
    }
}

impl Mipmap {
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format(&self) -> crate::TextureFormat {
        self.format.into()
    }

    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size(&self) -> (u32, u32) {
        self.base_size
    }

    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn levels(&self) -> &[Vec<u8>] {
        &self.levels
    }

    #[lsp_doc("docs/api/texture/mipmap/level.md")]
    pub fn level(&self, index: usize) -> Result<&[u8], TextureError> {
        self.levels.get(index).map(Vec::as_slice).ok_or_else(|| {
            TextureError::InvalidInput(format!(
                "level {index} out of range; the chain has {} levels",
                self.levels.len()
            ))
        })
    }

    #[lsp_doc("docs/api/texture/mipmap/count.md")]
    pub fn count(&self) -> usize {
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
/// A string is a URL when it starts with `http://` or `https://`, and a file path otherwise.
impl From<&str> for TextureData {
    fn from(source: &str) -> Self {
        if source.starts_with("http://") || source.starts_with("https://") {
            TextureData::Url(source.to_string())
        } else {
            TextureData::Path(std::path::PathBuf::from(source))
        }
    }
}
impl From<String> for TextureData {
    fn from(source: String) -> Self {
        TextureData::from(source.as_str())
    }
}
impl From<&String> for TextureData {
    fn from(source: &String) -> Self {
        TextureData::from(source.as_str())
    }
}
impl From<&Texture> for TextureData {
    fn from(t: &Texture) -> Self {
        TextureData::CloneOf(t.clone())
    }
}
impl From<Mipmap> for TextureData {
    fn from(chain: Mipmap) -> Self {
        TextureData::Prepared(chain)
    }
}

/// Uniffi mirror of [`TextureData`], which it converts into.
///
/// It leaves out `Empty`. Paths travel as `String`, handles as `Arc`.
///
/// The Rust core stays on `TextureData`. This enum exists only so the mobile
/// bindings can carry a concrete variant. Web and Python use their own dispatch
/// (`TryFrom<JsValue>` and Python kwargs) and never see this type.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum TextureInputMobile {
    Bytes(Vec<u8>),
    Path(String),
    Url(String),
    CloneOf(std::sync::Arc<Texture>),
    Prepared(std::sync::Arc<Mipmap>),
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

/// A texture source decoded from JS.
/// `ImageData` and canvases carry their pixel size in `options.size`,
/// so their raw RGBA bytes upload without a size argument.
#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureInput {
    type Error = crate::texture::error::TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, ArrayBuffer, Uint8Array, Uint8ClampedArray};
        use wasm_bindgen::JsCast;

        let pixels = |bytes: Vec<u8>, width: u32, height: u32| TextureInput {
            data: TextureData::Bytes(bytes),
            options: TextureOptions::from(Size::from([width, height])),
        };

        // Case: Mipmap handle (built off-thread via Mipmap.build).
        // Detected through the branded __fc_kind + __wbg_ptr anchor so it
        // survives bundler renaming. Must come before the Uint8Array / object
        // cases because wasm-bindgen handles also satisfy `is_object()`.
        if let Ok(chain) = Mipmap::try_from(value) {
            return Ok(TextureData::Prepared(chain).into());
        }

        // Case: an existing Texture handle, shared as is
        if let Ok(texture) = Texture::try_from(value) {
            return Ok(TextureData::CloneOf(texture).into());
        }

        // Case: Uint8Array (fast path)
        if let Some(u8a) = value.dyn_ref::<Uint8Array>() {
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes).into());
        }

        // Case: Uint8ClampedArray (ImageData.data)
        if let Some(u8c) = value.dyn_ref::<Uint8ClampedArray>() {
            let mut bytes = vec![0u8; u8c.length() as usize];
            u8c.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes).into());
        }

        // Case: ArrayBuffer
        if let Some(buf) = value.dyn_ref::<ArrayBuffer>() {
            // Guard against detached buffers via byte_length() == 0; safe fallback for empty too.
            if buf.byte_length() == 0 {
                return Ok(TextureData::Bytes(Vec::new()).into());
            }
            let u8a = Uint8Array::new(buf);
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return Ok(TextureData::Bytes(bytes).into());
        }

        // Case: ImageData -> its RGBA pixels and size
        if let Some(image_data) = value.dyn_ref::<web_sys::ImageData>() {
            let bytes = image_data.data().0;
            return Ok(pixels(bytes, image_data.width(), image_data.height()));
        }

        // Case: HTMLCanvasElement -> raw pixel bytes via 2D context
        if let Some(canvas) = value.dyn_ref::<web_sys::HtmlCanvasElement>()
            && let Ok(Some(ctx_js)) = canvas.get_context("2d")
            && let Ok(ctx) = ctx_js.dyn_into::<web_sys::CanvasRenderingContext2d>()
        {
            let (width, height) = (canvas.width(), canvas.height());
            let image = ctx.get_image_data(0.0, 0.0, width as f64, height as f64)?;
            return Ok(pixels(image.data().0, width, height));
        }

        // Case: OffscreenCanvas -> raw pixel bytes via 2D context
        if let Some(canvas) = value.dyn_ref::<web_sys::OffscreenCanvas>()
            && let Ok(Some(ctx_js)) = canvas.get_context("2d")
            && let Ok(ctx) = ctx_js.dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
        {
            let (width, height) = (canvas.width(), canvas.height());
            let image = ctx.get_image_data(0.0, 0.0, width as f64, height as f64)?;
            return Ok(pixels(image.data().0, width, height));
        }

        // Case: HTMLImageElement -> pass through as URL
        if let Some(img) = value.dyn_ref::<web_sys::HtmlImageElement>() {
            return Ok(TextureData::Url(img.src()).into());
        }

        // Case: Plain JS Array of numbers
        if Array::is_array(value) {
            let bytes = Array::from(value)
                .iter()
                .map(|item| js_to_byte(&item))
                .collect::<Result<Vec<u8>, TextureError>>()?;
            return Ok(TextureData::Bytes(bytes).into());
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
            return Ok(TextureData::Url(url).into());
        }

        Err(crate::texture::error::TextureError::Error(
            "Unsupported input for TextureInput".into(),
        ))
    }
}

/// One item of a JS byte list: a whole number from 0 to 255.
#[cfg(wasm)]
fn js_to_byte(value: &wasm_bindgen::JsValue) -> Result<u8, TextureError> {
    match value.as_f64() {
        Some(number) if number.fract() == 0.0 && (0.0..=255.0).contains(&number) => {
            Ok(number as u8)
        }
        _ => Err(TextureError::InvalidInput(format!(
            "a byte list takes whole numbers from 0 to 255, got {value:?}"
        ))),
    }
}

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureData {
    type Error = crate::texture::error::TextureError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        TextureInput::try_from(value).map(|input| input.data)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object, get_all, eq, hash, frozen))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextureId {
    /// Names a texture while it stays registered, in the `Renderer` that made it.
    /// A Material map's id names the copy in each `Renderer` that loaded that map.
    pub id: u64,
}

impl TextureId {
    /// Takes a new id from the counter that registrations and Material maps share,
    /// so no two textures in the process ever get the same one.
    pub(crate) fn create() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl From<u64> for TextureId {
    fn from(value: u64) -> Self {
        TextureId { id: value }
    }
}

impl From<&TextureId> for TextureId {
    fn from(value: &TextureId) -> Self {
        *value
    }
}

impl From<TextureId> for u64 {
    fn from(value: TextureId) -> Self {
        value.id
    }
}

crate::impl_fc_kind!(TextureId, "TextureId");
crate::impl_js_bridge!(TextureId, crate::texture::TextureError);

#[cfg(wasm)]
pub(crate) fn js_to_texture_id(
    value: &wasm_bindgen::JsValue,
) -> Result<TextureId, crate::texture::TextureError> {
    let invalid = || TextureError::InvalidInput(format!("not a texture id: {value:?}"));
    if let Some(number) = value.as_f64() {
        if number.fract() != 0.0 || !(0.0..=u64::MAX as f64).contains(&number) {
            return Err(invalid());
        }
        return Ok(TextureId { id: number as u64 });
    }
    if value.is_bigint() {
        return u64::try_from(value.clone())
            .map(TextureId::from)
            .map_err(|_| invalid());
    }
    TextureId::try_from(value)
}

#[cfg(wasm)]
/// The bytes of a JS source for a texture of `format` and, when known, `size`,
/// with the size the source brings. An `ImageData` or canvas holds RGBA pixels,
/// like a decoded image, so it gets the checks and the BGRA swap `createTexture` gives it.
pub(crate) fn js_to_texture_bytes(
    value: &wasm_bindgen::JsValue,
    size: Option<Size>,
    format: wgpu::TextureFormat,
) -> Result<(Vec<u8>, Option<Size>), crate::texture::TextureError> {
    let TextureInput { data, options } = TextureInput::try_from(value)?;
    let TextureData::Bytes(mut bytes) = data else {
        return Err(TextureError::Error("Expected raw byte data".into()));
    };
    if let Some(own) = options.size {
        check_pixels(own, size, format)?;
        swap_bgra(&mut bytes, format);
    }
    Ok((bytes, options.size))
}

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/texture/texture/texture.md")]
pub struct Texture {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) registration: Arc<Registration>,
}

crate::impl_fc_kind!(Texture, "Texture");

impl Texture {
    /// Creates a new Texture from a RenderContext.
    pub(crate) fn new(context: Arc<RenderContext>, registration: Arc<Registration>) -> Self {
        Self {
            context,
            registration,
        }
    }

    /// The texture this handle names now. A resize swaps it for every handle.
    pub(crate) fn object(&self) -> Arc<TextureObject> {
        self.registration.object()
    }

    /// Return the stable TextureId for this texture.
    /// The id is valid within the Renderer that created it.
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id(&self) -> &TextureId {
        self.registration.id()
    }

    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size(&self) -> crate::Size {
        self.object().size()
    }

    #[lsp_doc("docs/api/texture/texture/aspect.md")]
    pub fn aspect(&self) -> f32 {
        self.object().aspect()
    }

    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    pub fn set_sampler_options(&self, options: SamplerOptions) {
        self.registration
            .apply(|object| object.set_sampler_options(&self.context.device, options));
    }

    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write(&self, data: &[u8]) -> Result<(), TextureError> {
        write::write(self, data, None)
    }

    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    pub fn write_region<R>(&self, data: &[u8], region: R) -> Result<(), TextureError>
    where
        R: TryInto<TextureRegion>,
        TextureError: From<R::Error>,
    {
        write::write(self, data, Some(region.try_into()?))
    }

    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        read::get_image(self).await
    }

    #[lsp_doc("docs/api/texture/texture/resize.md")]
    pub fn resize(&self, size: impl Into<Size>) -> Result<(), TextureError> {
        resize::resize(&self.context, &self.registration, size.into())
    }
}

// Metadata for textures parsed from shader source; users do not construct directly.
#[cfg_attr(python, pyo3::pyclass(from_py_object))]
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
        let meta = TextureMeta::with_id_only(*texture.id());
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
    /// Create a texture with an explicit usage mask, such as a storage texture.
    /// It fails with a typed error when the device can't make `format` with `usage`.
    pub fn new(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        options: SamplerOptions,
    ) -> Result<Self, TextureError> {
        check_format(context, format, usage, dimension(size))?;
        let descriptor = Self::texture_descriptor("Generic Texture", size, format, usage, 1, 1); // @TODO mip_count, sample_count
        let inner = context.device.create_texture(&descriptor);
        let size = inner.size();
        let sampler = create_sampler(&context.device, options);
        Ok(Self {
            inner,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
            usage,
        })
    }

    /// Builds a texture on `context` from `source`, which the call shares and never copies.
    ///
    /// A URL is fetched on the calling thread, and its bytes go to the worker without
    /// being kept. The decode and the GPU upload run on the background worker on native
    /// and inline on the web. `Renderer::create_texture` and each Material map upload call it.
    pub(crate) async fn from_source(
        context: Arc<RenderContext>,
        source: TextureSource,
    ) -> Result<Self, TextureError> {
        let options = source.options.clone();
        check_source(source.data.is_sized(), options.size)?;
        let usage = source_usage(options.usage)?;
        let fetched = match &*source.data {
            SourceData::Url(url) | SourceData::Ktx2Url(url) => {
                crate::net::fetch_bytes(url).await.map_err(|e| {
                    TextureError::CreateTextureError(format!("URL fetch failed: {e:?}"))
                })?
            }
            _ => Vec::new(),
        };

        crate::renderer::background::run(move || -> Result<Self, TextureError> {
            let context = context.as_ref();
            let object = match &*source.data {
                SourceData::Bytes(bytes) => Self::from_bytes(context, bytes, &options, usage),
                SourceData::Url(_) => Self::from_bytes(context, &fetched, &options, usage),
                SourceData::Ktx2Bytes(bytes) => ktx2_loader::from_ktx2_bytes(context, bytes, usage),
                SourceData::Ktx2Url(_) => ktx2_loader::from_ktx2_bytes(context, &fetched, usage),
                SourceData::Path(path) if ktx2_loader::is_ktx2_file(path) => {
                    read_ktx2_file(context, path, usage)
                }
                SourceData::Path(path) => {
                    let limit = context.device.limits().max_texture_dimension_2d;
                    let image = decode(&read_file(path)?, limit)?;
                    let format = pick_override(&options);
                    Self::from_image(context, &image, format, options.mipmaps, usage)
                }
                SourceData::Ktx2Path(path) => read_ktx2_file(context, path, usage),
                SourceData::Prepared(chain) => Self::from_chain(context, chain.clone(), usage),
            }?;
            object.set_sampler_options(&context.device, options.sampler);
            Ok(object)
        })
        .await
    }

    /// Builds a texture from `bytes`: a KTX2 container by its magic, raw pixels
    /// when `options` gives a size, or else an encoded image the worker decodes.
    fn from_bytes(
        context: &RenderContext,
        bytes: &[u8],
        options: &TextureOptions,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        if ktx2_loader::is_ktx2(bytes) {
            return ktx2_loader::from_ktx2_bytes(context, bytes, usage);
        }
        let format = pick_override(options);
        match options.size {
            Some(size) => {
                let format = format.unwrap_or_else(|| wgpu::TextureFormat::from(options.format));
                Self::from_raw_bytes(context, size.into(), format, bytes, options.mipmaps, usage)
            }
            None => {
                let limit = context.device.limits().max_texture_dimension_2d;
                let image = decode(bytes, limit)?;
                Self::from_image(context, &image, format, options.mipmaps, usage)
            }
        }
    }

    /// Creates a texture from raw pixel bytes with explicit size/format.
    /// The data layout is tightly packed with bytes_per_row = bpp * width.
    ///
    /// When `generate_mipmaps = true`, a full mip chain is built CPU-side via the
    /// dispatcher in [`write_levels`], for every format `supports_cpu_mipmaps`
    /// accepts, the float formats included. Other formats, and a 3D size, get one level.
    fn from_raw_bytes(
        context: &RenderContext,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        data: &[u8],
        generate_mipmaps: bool,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        check_format(context, format, usage, dimension(size))?;
        check_upload(format)?;
        check_extent(&context.device.limits(), size, dimension(size))?;
        let bpp = texel_size(format)?;

        let expected = (size.width as usize)
            .saturating_mul(size.height as usize)
            .saturating_mul(size.depth_or_array_layers as usize)
            .saturating_mul(bpp as usize);
        if data.len() != expected {
            let wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            } = size;
            return Err(TextureError::InvalidInput(format!(
                "raw bytes are {} long, and {width}x{height}x{depth} {format:?} takes {expected}",
                data.len()
            )));
        }

        // CPU mipmaps resample one slice, so a 3D texture keeps one level and every slice.
        let flat = size.depth_or_array_layers == 1;
        let want_mips = generate_mipmaps && flat && supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(size.width, size.height)
        } else {
            1
        };

        let descriptor =
            Self::texture_descriptor("Raw Texture", size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_levels(context, data, &texture, size, format, mip_level_count)?;

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
    /// `format_override = None` infers the format from `image.color()`, and any
    /// format outside `supports_cpu_mipmaps` is refused. The image is decoded again
    /// to match the format, so `R16Unorm` goes through `to_luma16()` and keeps 16 bits.
    /// Mips of a unorm format come from `image::imageops::resize` with the Triangle filter.
    /// The float formats take [`downsample_floats`] instead, since that resize clamps them to 0..1.
    fn from_image(
        context: &RenderContext,
        image: &DynamicImage,
        format_override: Option<wgpu::TextureFormat>,
        generate_mipmaps: bool,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        let label = "Source Texture from Loaded Image";
        let (width, height) = image.dimensions();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let format = format_override.unwrap_or_else(|| infer_format(image.color()));
        check_format(context, format, usage, dimension(size))?;
        check_upload(format)?;
        check_decode(format)?;
        check_extent(&context.device.limits(), size, dimension(size))?;

        let want_mips = generate_mipmaps && supports_cpu_mipmaps(format);
        let mip_level_count = if want_mips {
            mip_level_count_for(width, height)
        } else {
            1
        };
        let descriptor = Self::texture_descriptor(label, size, format, usage, 1, mip_level_count);
        let texture = context.device.create_texture(&descriptor);

        write_image_levels(context, image, &texture, format, mip_level_count)?;

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

    /// Create a TextureObject from a pre-built CPU mip chain. Pure GPU writes —
    /// no decode, no resize. Used by the `TextureInput::Prepared` path.
    fn from_chain(
        context: &RenderContext,
        chain: Mipmap,
        usage: wgpu::TextureUsages,
    ) -> Result<Self, TextureError> {
        let (w, h) = chain.base_size;
        if w == 0 || h == 0 {
            return Err(TextureError::CreateTextureError(
                "Mipmap size must be non-zero".into(),
            ));
        }
        if chain.levels.is_empty() {
            return Err(TextureError::CreateTextureError(
                "Mipmap must contain at least one level".into(),
            ));
        }
        let levels = chain.levels.as_slice();
        let format = chain.format;
        check_format(context, format, usage, wgpu::TextureDimension::D2)?;
        let bpp = texel_size(format)?;

        for (i, level_bytes) in levels.iter().enumerate() {
            let level_w = (w >> i).max(1);
            let level_h = (h >> i).max(1);
            let expected = (bpp as usize) * (level_w as usize) * (level_h as usize);
            if level_bytes.len() != expected {
                return Err(TextureError::CreateTextureError(format!(
                    "Mipmap level {} has {} bytes, expected {} ({}x{} @ {} bpp)",
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
        check_extent(&context.device.limits(), size, dimension(size))?;
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

    /// Creates a depth texture with the given MSAA `sample_count`. Pass
    /// `context.sample_count()` to inherit the renderer's negotiated count;
    /// pass `1` for a non-MSAA depth attachment.
    pub fn create_depth_texture(
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
            dimension: dimension(size),
            format,
            view_formats: &[],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = context.device.create_texture(&descriptor);
        let options = SamplerOptions {
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        };
        let sampler = create_sampler(&context.device, options);

        Self {
            inner: texture,
            size,
            sampler: RwLock::new(sampler),
            options: RwLock::new(options),
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    /// The sampler a `sampler` binding gets from this texture.
    /// It is the texture's own sampler when the binding's `comparison` matches the options,
    /// and otherwise a sampler built from the same options with the comparison switched on or off.
    pub fn sampler_for(&self, device: &wgpu::Device, comparison: bool) -> wgpu::Sampler {
        let options = self.options.read();
        if options.compare.is_some() == comparison {
            return self.sampler.read().clone();
        }
        let compare = comparison.then_some(CompareFunction::LessEqual);
        create_sampler(
            device,
            SamplerOptions {
                compare,
                ..*options
            },
        )
    }

    pub fn set_sampler_options(&self, device: &wgpu::Device, options: SamplerOptions) {
        let sampler = create_sampler(device, options);
        // Hold the options lock across both writes so `sampler_for` never sees a mismatched pair.
        let mut current = self.options.write();
        *self.sampler.write() = sampler;
        *current = options;
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

    /// This texture at `size`, with the same format, usage, sample count and
    /// sampler, and as many of its mip levels as fit. Its pixels start blank.
    pub(crate) fn resized(&self, context: &RenderContext, size: wgpu::Extent3d) -> Self {
        let label = match self.format.is_depth_stencil_format() {
            true => "Depth Texture",
            false => "Render Target Texture",
        };
        let levels = mip_level_count_for(size.width, size.height);
        let descriptor = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: self.inner.mip_level_count().min(levels),
            sample_count: self.inner.sample_count(),
            dimension: self.inner.dimension(),
            format: self.format,
            view_formats: &[],
            usage: self.usage,
        };
        let options = self.options.read();
        Self {
            inner: context.device.create_texture(&descriptor),
            size,
            sampler: RwLock::new(self.sampler.read().clone()),
            options: RwLock::new(*options),
            format: self.format,
            usage: self.usage,
        }
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
            dimension: dimension(size),
            format,
            view_formats: &[],
            usage,
        }
    }
}

fn read_ktx2_file(
    context: &RenderContext,
    path: &std::path::Path,
    usage: wgpu::TextureUsages,
) -> Result<TextureObject, TextureError> {
    let bytes = std::fs::read(path).map_err(|e| {
        TextureError::CreateTextureError(format!(
            "Failed to read KTX2 file {}: {}",
            path.display(),
            e
        ))
    })?;
    ktx2_loader::from_ktx2_bytes(context, &bytes, usage)
}

/// The format `options` names, or `None` for the default, which lets the image pick.
fn pick_override(options: &TextureOptions) -> Option<wgpu::TextureFormat> {
    match options.format {
        crate::TextureFormat::Rgba => None,
        format => Some(wgpu::TextureFormat::from(format)),
    }
}

/// Checks that a `size` comes only with raw pixel bytes, given whether the data is
/// `sized`, since a path, a URL, a KTX2 file and a mip chain each carry their own.
fn check_source(sized: bool, size: Option<Size>) -> Result<(), TextureError> {
    match sized && size.is_some() {
        true => Err(TextureError::InvalidInput(
            "a size describes raw pixel bytes; a path, URL, KTX2 file or Mipmap takes its size from the file".into(),
        )),
        false => Ok(()),
    }
}

/// Decodes an encoded image, its format guessed from the bytes, once each side
/// its header declares fits `limit`, so an image too large to upload is refused
/// before its pixels take any memory. `create_texture` and `Mipmap::build` share it.
fn decode(bytes: &[u8], limit: u32) -> Result<DynamicImage, TextureError> {
    let (width, height) = read_dimensions(bytes)?;
    check_size(width, height, limit)?;
    Ok(decode_pixels(bytes)?)
}

/// The width and height an encoded image's header declares.
fn read_dimensions(bytes: &[u8]) -> Result<(u32, u32), ImageFailure> {
    let format = image::guess_format(bytes)?;
    let reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    Ok(reader.into_dimensions()?)
}

/// The pixels of an encoded image, its format guessed from the bytes.
fn decode_pixels(bytes: &[u8]) -> Result<DynamicImage, ImageFailure> {
    let format = image::guess_format(bytes)?;
    let reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    Ok(reader.decode()?)
}

/// A failure of the image library, which `TextureError::MalformedImageError` carries as text.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(crate) struct ImageFailure(#[from] image::ImageError);

impl From<std::io::Error> for ImageFailure {
    fn from(error: std::io::Error) -> Self {
        ImageFailure(image::ImageError::IoError(error))
    }
}

impl From<ImageFailure> for TextureError {
    fn from(failure: ImageFailure) -> Self {
        TextureError::MalformedImageError(crate::shader::error::describe(&failure.0))
    }
}

/// The bytes of the file at `path`. A failed read is an image error, as `image::open` reports it.
fn read_file(path: &std::path::Path) -> Result<Vec<u8>, ImageFailure> {
    Ok(std::fs::read(path)?)
}

/// Usage of a texture made from source data.
/// It always holds `TEXTURE_BINDING | COPY_DST`, since the upload and sampling need them,
/// plus any bits from `TextureOptions::usage`, such as `COPY_SRC` for `read_texture`.
pub(crate) fn source_usage(extra: Option<u32>) -> Result<wgpu::TextureUsages, TextureError> {
    let required = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    let extra = extra.map(check_usage).transpose()?;
    Ok(required | extra.unwrap_or(wgpu::TextureUsages::empty()))
}

/// The dimension a texture of `size` gets: 2D with one layer, and 3D with more.
pub(crate) fn dimension(size: wgpu::Extent3d) -> wgpu::TextureDimension {
    match size.depth_or_array_layers {
        1 => wgpu::TextureDimension::D2,
        _ => wgpu::TextureDimension::D3,
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
/// `image` crate's `to_*` helpers, the float formats through `to_rgba32f`.
pub(crate) fn supports_cpu_mipmaps(format: wgpu::TextureFormat) -> bool {
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
            | wgpu::TextureFormat::Rgba16Float
            | wgpu::TextureFormat::Rgba32Float
    )
}

/// What the device lets a texture of `format` do, as wgpu checks it.
/// A backend below WebGPU, such as GLES or WebGL2, reports its own table,
/// and every other backend takes the WebGPU baseline for the device's features.
/// It reads the context's cached features and downlevel flags, so a check can run it each frame.
pub(crate) fn format_features(
    context: &RenderContext,
    format: wgpu::TextureFormat,
) -> wgpu::TextureFormatFeatures {
    let features = context.features;
    let below = !context
        .downlevel
        .contains(wgpu::DownlevelFlags::WEBGPU_TEXTURE_FORMAT_SUPPORT);
    let reports =
        features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) || below;
    match context.adapter.as_ref().filter(|_| reports) {
        Some(adapter) => adapter.get_texture_format_features(format),
        None => format.guaranteed_format_features(features),
    }
}

/// Checks the device makes `format` with `usage` and `dimension`,
/// so a texture wgpu would refuse never reaches `device.create_texture`.
pub(crate) fn check_format(
    context: &RenderContext,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    dimension: wgpu::TextureDimension,
) -> Result<(), TextureError> {
    check_feature(context.device.features(), format)?;
    let allowed = format_features(context, format).allowed_usages;
    check_support(allowed, format, usage, dimension)
}

/// Checks that the device's `features` hold every feature `format` needs.
pub(crate) fn check_feature(
    features: wgpu::Features,
    format: wgpu::TextureFormat,
) -> Result<(), TextureError> {
    let missing = format.required_features() - features;
    if missing.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = missing.iter_names().map(|(name, _)| name).collect();
    Err(TextureError::UnsupportedFormatForUsage {
        format: format!("{format:?}"),
        missing_feature: names.join(" | "),
    })
}

/// Checks that a copy can fill a texture of `format` with pixels.
/// wgpu takes no buffer copy into a depth format, so such a texture is drawn into instead.
pub(crate) fn check_upload(format: wgpu::TextureFormat) -> Result<(), TextureError> {
    match format.is_depth_stencil_format() {
        true => Err(TextureError::InvalidInput(format!(
            "{format:?} takes no uploaded pixels, render into it instead"
        ))),
        false => Ok(()),
    }
}

/// Checks that a decoded image converts into `format`,
/// one the CPU mipmap path fills. Integer formats take raw bytes.
pub(crate) fn check_decode(format: wgpu::TextureFormat) -> Result<(), TextureError> {
    match supports_cpu_mipmaps(format) {
        true => Ok(()),
        false => Err(TextureError::InvalidInput(format!(
            "can't decode an image into {format:?}, pass raw bytes with a size"
        ))),
    }
}

/// Checks `usage` against the usages `allowed` for `format`,
/// and that a depth format makes a 2D texture, the only kind it has.
pub(crate) fn check_support(
    allowed: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    dimension: wgpu::TextureDimension,
) -> Result<(), TextureError> {
    let missing = usage - allowed;
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter_names().map(|(name, _)| name).collect();
        return Err(TextureError::InvalidInput(format!(
            "a {format:?} texture can't take the usage {}",
            names.join(" | ")
        )));
    }
    if format.is_depth_stencil_format() && dimension != wgpu::TextureDimension::D2 {
        return Err(TextureError::InvalidInput(format!(
            "a {format:?} texture has one layer, so its depth must be 1"
        )));
    }
    Ok(())
}

/// Map an `image::ColorType` to a sensible default `wgpu::TextureFormat` for sampling.
pub(crate) fn infer_format(color: image::ColorType) -> wgpu::TextureFormat {
    match color {
        image::ColorType::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        image::ColorType::L8 => wgpu::TextureFormat::R8Unorm,
        image::ColorType::La8 => wgpu::TextureFormat::Rg8Unorm,
        image::ColorType::Rgb8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        image::ColorType::L16 => wgpu::TextureFormat::R16Unorm,
        image::ColorType::La16 => wgpu::TextureFormat::Rg16Unorm,
        image::ColorType::Rgb16 => wgpu::TextureFormat::Rgba16Unorm,
        image::ColorType::Rgba16 => wgpu::TextureFormat::Rgba16Unorm,
        image::ColorType::Rgb32F => wgpu::TextureFormat::Rgba16Float,
        image::ColorType::Rgba32F => wgpu::TextureFormat::Rgba16Float,
        _ => wgpu::TextureFormat::Rgba8UnormSrgb,
    }
}

/// Uploads `image` with its mips, decoded into the pixels `format` holds.
///
/// A format [`check_decode`] refuses is an error, as it is in `from_image`.
/// Each branch decodes it into the channels and bytes per pixel of `format`,
/// so a 16-bit grayscale PNG keeps its precision as `R16Unorm`, never `to_rgba8`.
/// A unorm format takes its mips from `image::imageops::resize` with the Triangle filter.
/// The float formats take [`downsample_floats`] instead, since that resize clamps them to 0..1.
pub(crate) fn write_image_levels(
    context: &RenderContext,
    image: &DynamicImage,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> Result<(), TextureError> {
    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            write_pixel_levels(
                context,
                decode_rgba8(image, format),
                texture,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::R8Unorm => {
            write_pixel_levels(context, image.to_luma8(), texture, mip_level_count, 1);
        }
        wgpu::TextureFormat::Rg8Unorm => {
            write_pixel_levels(context, image.to_luma_alpha8(), texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::R16Unorm => {
            write_pixel_levels(context, image.to_luma16(), texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::Rg16Unorm => {
            write_pixel_levels(
                context,
                image.to_luma_alpha16(),
                texture,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            write_pixel_levels(context, image.to_rgba16(), texture, mip_level_count, 8);
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            write_floats(
                context,
                image.to_rgba32f(),
                texture,
                mip_level_count,
                format,
                None,
            )?;
        }
        _ => check_decode(format)?,
    }
    Ok(())
}

/// Uploads `base` and the float levels below it, `count` in all, packed for `format`.
/// `raw`, the caller's own bytes for `base`, uploads as level 0 byte for byte.
fn write_floats(
    context: &RenderContext,
    base: image::Rgba32FImage,
    texture: &wgpu::Texture,
    count: u32,
    format: wgpu::TextureFormat,
    raw: Option<&[u8]>,
) -> Result<(), TextureError> {
    let size = texel_size(format)?;
    for (level, image) in downsample_floats(base, count as usize).enumerate() {
        let bytes = match (level, raw) {
            (0, Some(raw)) => std::borrow::Cow::Borrowed(raw),
            _ => std::borrow::Cow::Owned(pack_floats(image.as_raw(), format)),
        };
        let level = level as u32;
        write_level(context, texture, level, &bytes, image.dimensions(), size);
    }
    Ok(())
}

/// The bytes of f32 `values` in `format`: half floats saturated to their range
/// for `Rgba16Float`, and the values exactly as they are for `Rgba32Float`.
fn pack_floats(values: &[f32], format: wgpu::TextureFormat) -> Vec<u8> {
    match format {
        wgpu::TextureFormat::Rgba16Float => values
            .iter()
            .flat_map(|value| to_half(*value).to_le_bytes())
            .collect(),
        _ => bytemuck::cast_slice(values).to_vec(),
    }
}

/// The f32 values of tightly packed `Rgba16Float` or `Rgba32Float` bytes.
fn read_floats(bytes: &[u8], format: wgpu::TextureFormat) -> Vec<f32> {
    match format {
        wgpu::TextureFormat::Rgba16Float => {
            let half = |word: &[u8; 2]| half::f16::from_le_bytes(*word).to_f32();
            bytes.as_chunks::<2>().0.iter().map(half).collect()
        }
        _ => {
            let float = |word: &[u8; 4]| f32::from_le_bytes(*word);
            bytes.as_chunks::<4>().0.iter().map(float).collect()
        }
    }
}

/// `base` and the float levels below it, `count` in all, each half the size
/// of the one before, never smaller than 1x1, and averaged with [`halve`].
fn downsample_floats(
    base: image::Rgba32FImage,
    count: usize,
) -> impl Iterator<Item = image::Rgba32FImage> {
    std::iter::successors(Some(base), |level| Some(halve(level))).take(count)
}

/// `image` at half its size with a box filter, so a value past 0 to 1 keeps its size,
/// where the `image` crate's resize clamps a float channel to that range.
/// On an odd side the last texel joins the span before it, so every texel counts once.
fn halve(image: &image::Rgba32FImage) -> image::Rgba32FImage {
    let (width, height) = image.dimensions();
    let (half_width, half_height) = ((width / 2).max(1), (height / 2).max(1));
    // The source texels output texel `index` of `half` covers, out of `total`.
    let span = |index: u32, half: u32, total: u32| {
        let edge = |index: u32| (u64::from(index) * u64::from(total) / u64::from(half)) as u32;
        edge(index)..edge(index + 1)
    };
    image::Rgba32FImage::from_fn(half_width, half_height, |x, y| {
        let (mut sum, mut count) = ([0f64; 4], 0f64);
        for row in span(y, half_height, height) {
            for column in span(x, half_width, width) {
                let texel = image.get_pixel(column, row).0;
                for (total, value) in sum.iter_mut().zip(texel) {
                    *total += f64::from(value);
                }
                count += 1.0;
            }
        }
        image::Rgba(sum.map(|total| (total / count) as f32))
    })
}

/// `value` as a half float, saturated to the half float range,
/// so a value past 65504 stays 65504 where a plain conversion gives infinity.
fn to_half(value: f32) -> half::f16 {
    let limit = half::f16::MAX.to_f32();
    half::f16::from_f32(value.clamp(-limit, limit))
}

/// The texel bytes and size of a `uint8` array of `shape` for `format`.
/// Channels that match the format's bytes per pixel pass as they are, and
/// 1 or 3 channels gain an opaque alpha for an 8-bit RGBA or BGRA format.
/// The shape, `(h, w)` or `(h, w, channels)`, is checked before `read` collects the bytes.
#[cfg(any(python, test))]
pub(crate) fn array_pixels(
    shape: &[usize],
    read: impl FnOnce() -> Vec<u8>,
    format: wgpu::TextureFormat,
) -> Result<(Vec<u8>, Size), TextureError> {
    let invalid = || {
        TextureError::InvalidInput(format!(
            "an ndarray of shape {shape:?} doesn't fit a {format:?} texture"
        ))
    };
    let (height, width, channels) = match *shape {
        [height, width] => (height, width, 1),
        [height, width, channels] => (height, width, channels),
        _ => return Err(invalid()),
    };
    let side = |value: usize| u32::try_from(value).ok().ok_or_else(invalid);
    let size = Size::new(side(width)?, side(height)?, None);
    if channels == texel_size(format)? as usize {
        return Ok((read(), size));
    }
    if !holds_rgba8(format) || !matches!(channels, 1 | 3) {
        return Err(invalid());
    }
    let length = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(invalid)?;
    let mut pixels = Vec::with_capacity(length);
    for texel in read().chunks_exact(channels) {
        match *texel {
            [value] => pixels.extend_from_slice(&[value, value, value, 255]),
            [red, green, blue] => pixels.extend_from_slice(&[red, green, blue, 255]),
            _ => return Err(invalid()),
        }
    }
    Ok((pixels, size))
}

/// Checks a `size` and `format` for a source with RGBA8 pixels at the size `own`,
/// such as a JavaScript `ImageData` or canvas. A different size, or a
/// format other than 8-bit RGBA or BGRA, would misread the pixels it holds.
#[cfg(any(wasm, test))]
pub(crate) fn check_pixels(
    own: Size,
    size: Option<Size>,
    format: wgpu::TextureFormat,
) -> Result<(), TextureError> {
    let flat = |size: Size| (size.width, size.height, size.depth.unwrap_or(1));
    if let Some(size) = size.filter(|size| flat(*size) != flat(own)) {
        return Err(TextureError::InvalidInput(format!(
            "size {size:?} differs from the source's {}x{}",
            own.width, own.height
        )));
    }
    match holds_rgba8(format) {
        true => Ok(()),
        false => Err(TextureError::InvalidInput(format!(
            "the source holds RGBA8 pixels, which a {format:?} texture can't take"
        ))),
    }
}

/// True for the formats that store 8-bit RGBA or BGRA texels.
#[cfg(any(python, wasm, test))]
fn holds_rgba8(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
    )
}

/// Swaps red and blue in 4-byte `pixels` when `format` stores BGRA.
/// The swap is its own inverse, so it turns RGBA into BGRA and back.
pub(crate) fn swap_bgra(pixels: &mut [u8], format: wgpu::TextureFormat) {
    if matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
    }
}

/// The image as 4-byte pixels in the channel order of `format`, blue first for BGRA.
fn decode_rgba8(image: &DynamicImage, format: wgpu::TextureFormat) -> image::RgbaImage {
    let mut pixels = image.to_rgba8();
    swap_bgra(&mut pixels, format);
    pixels
}

/// Wrap raw pixel bytes (already laid out for `format` at `width x height`)
/// into a `DynamicImage` so [`build_mip_chain`] can downsample them with
/// the same Triangle filter used by the inline path. Mirrors the format
/// dispatch in [`write_levels`] but produces an in-memory image
/// instead of writing to the GPU.
pub(crate) fn bytes_as_image(
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
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            // The image holds RGBA, so BGRA bytes swap here and back when the chain is built.
            let mut pixels = bytes[..pixel_count * 4].to_vec();
            swap_bgra(&mut pixels, format);
            DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, pixels)
                    .ok_or_else(|| wrap_err("RgbaImage"))?,
            )
        }
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
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            let length = pixel_count * texel_size(format)? as usize;
            let values = read_floats(&bytes[..length], format);
            DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, values)
                    .ok_or_else(|| wrap_err("Rgba32F"))?,
            )
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "bytes_as_image: format {:?} is not supported",
                format
            )));
        }
    })
}

/// Pure-CPU mip chain builder used by [`Mipmap::build`]. Mirrors the format
/// dispatch in [`write_image_levels`] but buffers each level's bytes into a
/// `Vec<u8>` instead of writing to a wgpu texture. Caller must have validated
/// that `format` is in [`supports_cpu_mipmaps`].
pub(crate) fn build_mip_chain(
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
            downsample_levels_to_bytes(decode_rgba8(image, format), level_count)
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
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            downsample_floats(image.to_rgba32f(), level_count)
                .map(|level| pack_floats(level.as_raw(), format))
                .collect()
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "build_mip_chain: format {:?} is not supported",
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
    downsample(base, level_count)
        .map(|level| bytemuck::cast_slice(level.as_raw()).to_vec())
        .collect()
}

/// `base` and the levels below it, `count` in all, each half the size of the
/// one before with the Triangle filter, and never smaller than 1x1.
/// The filter clamps each channel to its type's range, which suits the integer
/// formats, so the float formats go through [`downsample_floats`] instead.
fn downsample<P>(
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    count: usize,
) -> impl Iterator<Item = image::ImageBuffer<P, Vec<P::Subpixel>>>
where
    P: image::Pixel + 'static,
    P::Subpixel: 'static,
{
    let half = |level: &image::ImageBuffer<P, Vec<P::Subpixel>>| {
        let (width, height) = ((level.width() / 2).max(1), (level.height() / 2).max(1));
        let filter = image::imageops::FilterType::Triangle;
        Some(image::imageops::resize(level, width, height, filter))
    };
    std::iter::successors(Some(base), half).take(count)
}

/// Wrap pre-decoded raw bytes in the right `ImageBuffer` for `format`, then upload
/// (and downsample for higher mip levels). Mirrors [`write_image_levels`] but starts
/// from a `&[u8]` buffer the caller has already laid out tightly. For 16-bit formats,
/// the byte slice is converted to `Vec<u16>` via `from_le_bytes` (alignment-safe and
/// matches WebGPU's little-endian element ordering).
pub(crate) fn write_levels(
    context: &RenderContext,
    bytes: &[u8],
    texture: &wgpu::Texture,
    full_size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> Result<(), TextureError> {
    if mip_level_count == 1 {
        let bpp = texel_size(format)?;
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
            write_pixel_levels(context, base, texture, mip_level_count, 4);
        }
        wgpu::TextureFormat::R8Unorm => {
            let base: image::GrayImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count].to_vec())
                    .ok_or_else(|| wrap_err("GrayImage"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 1);
        }
        wgpu::TextureFormat::Rg8Unorm => {
            let base: image::GrayAlphaImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count * 2].to_vec())
                    .ok_or_else(|| wrap_err("GrayAlphaImage"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::R16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 2]);
            let base: image::ImageBuffer<image::Luma<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Luma16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::Rg16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 4]);
            let base: image::ImageBuffer<image::LumaA<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("LumaA16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 4);
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 8]);
            let base: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Rgba16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 8);
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            let length = pixel_count * texel_size(format)? as usize;
            let raw = &bytes[..length];
            let base = image::Rgba32FImage::from_raw(w, h, read_floats(raw, format))
                .ok_or_else(|| wrap_err("Rgba32F"))?;
            write_floats(context, base, texture, mip_level_count, format, Some(raw))?;
        }
        _ => {
            // Caller should have caught this via supports_cpu_mipmaps;
            // be defensive and write a single level.
            let bpp = texel_size(format)?;
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
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect()
}

/// Generic worker: upload `base` as mip 0, then iteratively downsample with the
/// Triangle filter until the chain is full. Works for any pixel type the `image`
/// crate's `imageops::resize` supports (Rgba8, Luma8, LumaA8, Luma16, LumaA16,
/// Rgba16). Pixel data is reinterpreted to bytes via `bytemuck::cast_slice`,
/// which is a no-op for `u8` subpixels and a little-endian byte view for wider ones.
/// All FragmentColor target platforms are little-endian, which matches WebGPU's
/// expected byte order.
pub(crate) fn write_pixel_levels<P>(
    context: &RenderContext,
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    texture: &wgpu::Texture,
    mip_level_count: u32,
    bpp: u32,
) where
    P: image::Pixel + 'static,
    P::Subpixel: bytemuck::NoUninit + 'static,
{
    for (level, image) in downsample(base, mip_level_count as usize).enumerate() {
        let bytes: &[u8] = bytemuck::cast_slice(image.as_raw());
        write_level(
            context,
            texture,
            level as u32,
            bytes,
            image.dimensions(),
            bpp,
        );
    }
}

/// Uploads the tightly packed `bytes` of one mip `level`, `size` texels across and down.
fn write_level(
    context: &RenderContext,
    texture: &wgpu::Texture,
    level: u32,
    bytes: &[u8],
    (width, height): (u32, u32),
    bpp: u32,
) {
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
            bytes_per_row: Some(bpp * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

/// The bytes a texel of `format` takes in a copy, one texel at a time.
/// A block-compressed format, and a depth format with no copy size, are refused.
pub(crate) fn texel_size(format: wgpu::TextureFormat) -> Result<u32, TextureError> {
    match (format.block_dimensions(), format.block_copy_size(None)) {
        ((1, 1), Some(size)) => Ok(size),
        _ => Err(TextureError::InvalidInput(format!(
            "can't copy pixels of {format:?}, write and read take formats with a fixed size per pixel"
        ))),
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Texture, TextureError);

#[cfg(test)]
mod tests {
    use super::*;

    /// Encodes `image` as PNG bytes, the way a file on disk would hold it.
    fn encode_png(image: &DynamicImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode png");
        bytes
    }

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

    // Story: a string is a URL with an http(s) scheme and a file path otherwise.
    #[test]
    fn texture_input_from_strings() {
        assert!(matches!(
            TextureData::from("https://example.com/logo.png"),
            TextureData::Url(url) if url == "https://example.com/logo.png"
        ));
        assert!(matches!(
            TextureData::from(String::from("assets/logo.png")),
            TextureData::Path(path) if path == std::path::Path::new("assets/logo.png")
        ));
        let input = TextureInput::from(("logo.png", [2u32, 2]));
        assert!(matches!(input.data, TextureData::Path(_)));
        assert_eq!(input.options.size, Some(Size::from([2u32, 2])));
    }

    // Story: Mipmap::level returns one level and errors past the end.
    #[test]
    fn mipmap_level_indexes_the_chain() {
        let pixels = vec![9u8; 4 * 4 * 4];
        let chain = Mipmap::build((
            pixels.as_slice(),
            crate::TextureFormat::Rgba8Unorm,
            [4u32, 4],
        ))
        .expect("chain");
        assert_eq!(chain.level(0).expect("level 0").len(), 4 * 4 * 4);
        assert_eq!(chain.level(2).expect("level 2").len(), 4);
        assert!(matches!(chain.level(3), Err(TextureError::InvalidInput(_))));
    }

    // Story: an ndarray's bytes pass as they are when its channels match the
    // format, gain alpha for an 8-bit RGBA format, and anything else, a shape
    // past u32 included, is refused before its bytes are read.
    #[test]
    fn array_pixels_fit_the_format() {
        let gray = [1u8, 2, 3, 4];
        let read = || gray.to_vec();
        let (bytes, size) =
            array_pixels(&[2, 2], read, wgpu::TextureFormat::R8Unorm).expect("gray as R8");
        assert_eq!((bytes, size), (gray.to_vec(), Size::new(2, 2, None)));
        let (bytes, _) =
            array_pixels(&[2, 2], read, wgpu::TextureFormat::Rgba8Unorm).expect("gray as RGBA");
        assert_eq!(&bytes[..8], &[1, 1, 1, 255, 2, 2, 2, 255]);
        let (bytes, size) = array_pixels(&[1, 2, 2], read, wgpu::TextureFormat::Rg8Unorm)
            .expect("two channels as Rg8");
        assert_eq!((bytes, size), (gray.to_vec(), Size::new(2, 1, None)));

        let unread = || -> Vec<u8> { panic!("the bytes of a refused shape are never read") };
        for (shape, format) in [
            (vec![2, 2], wgpu::TextureFormat::Rgba32Float),
            (vec![2, 2, 3], wgpu::TextureFormat::Rg16Unorm),
            (vec![2, 2, 5], wgpu::TextureFormat::Rgba8Unorm),
            (vec![2, 2, 2, 2], wgpu::TextureFormat::Rgba8Unorm),
            (vec![1, (1 << 32) + 4, 3], wgpu::TextureFormat::Rgba8Unorm),
        ] {
            assert!(
                matches!(
                    array_pixels(&shape, unread, format),
                    Err(TextureError::InvalidInput(_))
                ),
                "{shape:?} as {format:?}"
            );
        }
    }

    // Story: options for a source with its own RGBA8 pixels keep its size and an
    // 8-bit RGBA or BGRA format, and anything else is refused, where it misread them.
    #[test]
    fn check_pixels_keeps_the_source_layout() {
        let own = Size::new(2, 1, None);
        for format in [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
        ] {
            check_pixels(own, None, format).expect("an RGBA8 format");
        }
        check_pixels(
            own,
            Some(Size::new(2, 1, Some(1))),
            wgpu::TextureFormat::Rgba8Unorm,
        )
        .expect("the source's own size");
        for (size, format) in [
            (Some(Size::new(1, 1, None)), wgpu::TextureFormat::Rgba8Unorm),
            (None, wgpu::TextureFormat::R8Unorm),
            (None, wgpu::TextureFormat::Rgba32Float),
        ] {
            assert!(
                matches!(
                    check_pixels(own, size, format),
                    Err(TextureError::InvalidInput(_))
                ),
                "{size:?} {format:?}"
            );
        }
    }

    // Story: Create a texture from encoded image bytes and verify aspect.
    #[test]
    fn encoded_image_aspect_smoke() {
        pollster::block_on(async move {
            // 2x1 image in memory
            let img = image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_vec(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]).unwrap(),
            );

            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::Bytes(encode_png(&img)))
                .await
                .expect("create texture from encoded image");
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
    fn encoded_image_creates_mipmap_chain_by_default() {
        pollster::block_on(async move {
            // 8x8 solid color image — generates 4 mip levels (8, 4, 2, 1).
            let pixels = vec![200u8; 8 * 8 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
            let png = encode_png(&img);
            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::Bytes(png.clone()))
                .await
                .expect("create texture with mipmaps");
            assert_eq!(tex.object().inner.mip_level_count(), 4);

            // Opt-out: only mip 0
            let opts = TextureOptions {
                mipmaps: false,
                ..Default::default()
            };
            let tex_no_mip = r
                .create_texture((TextureData::Bytes(png), opts))
                .await
                .expect("create texture without mipmaps");
            assert_eq!(tex_no_mip.object().inner.mip_level_count(), 1);
        });
    }

    // Story: Format override is honored on the encoded image arm — same RGBA bytes,
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
                .create_texture((TextureData::Bytes(encode_png(&img)), opts))
                .await
                .expect("create texture with explicit linear format");
            assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba8Unorm);
            assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
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
                .create_texture(TextureData::Bytes(encode_png(&img)))
                .await
                .expect("create R16Unorm texture from Luma16 image");
            assert_eq!(tex.object().format, wgpu::TextureFormat::R16Unorm);
            assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
        });
    }

    /// The texels of mip `level` of `texture`, copied straight from the GPU.
    fn read_level(texture: &Texture, level: u32) -> Vec<u8> {
        let object = texture.object();
        let context = &texture.context;
        let bpp = texel_size(object.format).expect("texel size");
        let (width, height) = (
            (object.size.width >> level).max(1),
            (object.size.height >> level).max(1),
        );
        let (row, padded) = (width * bpp, (width * bpp).div_ceil(256) * 256);
        let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Level readback"),
            size: u64::from(padded * height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = context.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &object.inner,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        context.queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        context
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("poll");
        let view = slice.get_mapped_range().expect("mapped range");
        let rows = view.chunks(padded as usize).take(height as usize);
        rows.flat_map(|line| line[..row as usize].to_vec())
            .collect()
    }

    /// The bytes of `texels` in the float `format`.
    fn encode_floats(texels: &[f32], format: crate::TextureFormat) -> Vec<u8> {
        match format {
            crate::TextureFormat::Rgba16Float => texels
                .iter()
                .flat_map(|value| half::f16::from_f32(*value).to_le_bytes())
                .collect(),
            _ => texels
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect(),
        }
    }

    /// The first texel of `bytes`, read as the float `format`.
    fn read_texel(bytes: &[u8], format: crate::TextureFormat) -> Vec<f32> {
        read_floats(bytes, format.into())[..4].to_vec()
    }

    // Story: a float mip level keeps values past 0 to 1, from raw bytes, a whole
    // write, Mipmap::build and an HDR decode, where the image crate's resize
    // clamped every level below the first to that range.
    #[test]
    fn float_levels_keep_their_range() {
        pollster::block_on(async move {
            let r = crate::Renderer::new();
            let sky = [10.0, -2.0, 0.5, 1.0];
            let night = [20.0, -4.0, 0.25, 1.0];
            for format in [
                crate::TextureFormat::Rgba16Float,
                crate::TextureFormat::Rgba32Float,
            ] {
                let bytes = encode_floats(&sky.repeat(4), format);
                let options = TextureOptions {
                    size: Some([2u32, 2].into()),
                    format,
                    ..Default::default()
                }
                .with_usage(crate::TextureUsage::COPY_SRC);
                let tex = r
                    .create_texture((bytes.clone(), options))
                    .await
                    .expect("raw float texture");
                assert_eq!(tex.object().inner.mip_level_count(), 2);
                assert_eq!(read_texel(&read_level(&tex, 1), format), sky, "{format:?}");

                tex.write(&encode_floats(&night.repeat(4), format))
                    .expect("a whole write");
                let level = read_level(&tex, 1);
                assert_eq!(read_texel(&level, format), night, "{format:?} write");

                let chain = Mipmap::build((bytes.as_slice(), format, [2u32, 2])).expect("chain");
                let level = chain.level(1).expect("level 1");
                assert_eq!(read_texel(level, format), sky, "{format:?} Mipmap");
            }

            let hdr_sky = image::Rgb32FImage::from_pixel(2, 2, image::Rgb([10.0, 5.0, 2.5]));
            let mut hdr = Vec::new();
            DynamicImage::ImageRgb32F(hdr_sky)
                .write_to(&mut std::io::Cursor::new(&mut hdr), image::ImageFormat::Hdr)
                .expect("encode hdr");
            let options = TextureOptions::default().with_usage(crate::TextureUsage::COPY_SRC);
            let tex = r
                .create_texture((TextureData::Bytes(hdr), options))
                .await
                .expect("an HDR file");
            let format = crate::TextureFormat::Rgba16Float;
            let texel = read_texel(&read_level(&tex, 1), format);
            assert!(
                (texel[0] - 10.0).abs() < 0.2,
                "HDR level 1 keeps 10, got {texel:?}"
            );
        });
    }

    // Story: raw half floats upload as given at level 0, an infinity and a NaN
    // payload included, and only the levels below it are resampled.
    #[test]
    fn raw_half_floats_keep_level_zero() {
        pollster::block_on(async move {
            let words: [u16; 16] = [0x7C00, 0x7E01, 0x3C00, 0x3C00]
                .repeat(4)
                .try_into()
                .unwrap();
            let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            let format = crate::TextureFormat::Rgba16Float;
            let options = TextureOptions {
                size: Some([2u32, 2].into()),
                format,
                ..Default::default()
            }
            .with_usage(crate::TextureUsage::COPY_SRC);
            let r = crate::Renderer::new();
            let tex = r
                .create_texture((bytes.clone(), options))
                .await
                .expect("raw half floats");
            assert_eq!(tex.object().inner.mip_level_count(), 2);
            assert_eq!(read_level(&tex, 0), bytes, "create_texture keeps level 0");
            tex.write(&bytes).expect("a whole write");
            assert_eq!(read_level(&tex, 0), bytes, "write keeps level 0");
            let chain = Mipmap::build((bytes.as_slice(), format, [2u32, 2])).expect("chain");
            assert_eq!(
                chain.level(0).expect("level 0"),
                bytes,
                "Mipmap keeps level 0"
            );
        });
    }

    // Story: on an odd side the last texel joins the span before it,
    // so every texel of a 5x3 or a 3x1 image counts once in the level below.
    #[test]
    fn halve_averages_odd_sides() {
        let gradient = |width, height| {
            image::Rgba32FImage::from_fn(width, height, |column, row| {
                image::Rgba([(column + 10 * row) as f32; 4])
            })
        };
        let level = halve(&gradient(5, 3));
        assert_eq!(level.dimensions(), (2, 1));
        assert_eq!(level.get_pixel(0, 0).0, [10.5; 4], "6 texels on the left");
        assert_eq!(level.get_pixel(1, 0).0, [13.0; 4], "9 texels on the right");
        let level = halve(&gradient(3, 1));
        assert_eq!(level.dimensions(), (1, 1));
        assert_eq!(level.get_pixel(0, 0).0, [1.0; 4], "all 3 texels");
    }

    // Story: an HDR file and a float format get the full mip chain every other
    // source image gets by default, in create_texture and in Mipmap::build.
    #[test]
    fn float_images_get_a_mip_chain() {
        pollster::block_on(async move {
            let sky = image::Rgb32FImage::from_pixel(8, 8, image::Rgb([2.0, 1.0, 0.5]));
            let mut hdr = Vec::new();
            DynamicImage::ImageRgb32F(sky)
                .write_to(&mut std::io::Cursor::new(&mut hdr), image::ImageFormat::Hdr)
                .expect("encode hdr");
            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::Bytes(hdr))
                .await
                .expect("create texture from an HDR file");
            assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba16Float);
            assert_eq!(tex.object().inner.mip_level_count(), 4); // 8, 4, 2, 1

            let pixels = vec![128u8; 8 * 8 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
            let png = encode_png(&img);
            let opts = TextureOptions::from(crate::TextureFormat::Rgba32Float);
            let tex = r
                .create_texture((TextureData::Bytes(png.clone()), opts))
                .await
                .expect("create Rgba32Float texture from a PNG");
            assert_eq!(tex.object().inner.mip_level_count(), 4);

            let chain = Mipmap::build((png.as_slice(), crate::TextureFormat::Rgba16Float))
                .expect("a half float chain");
            let sizes: Vec<usize> = chain.levels().iter().map(Vec::len).collect();
            assert_eq!(sizes, [8 * 64, 8 * 16, 8 * 4, 8]);
            let half = half::f16::from_le_bytes([chain.levels()[3][0], chain.levels()[3][1]]);
            assert!(
                (half.to_f32() - 128.0 / 255.0).abs() < 0.01,
                "level 3 keeps the value"
            );
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
                .create_texture(TextureData::Bytes(encode_png(&img)))
                .await
                .expect("create R8Unorm texture from Luma8 image");
            assert_eq!(tex.object().format, wgpu::TextureFormat::R8Unorm);
            assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
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
        let chain = Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
            .expect("prepare ok");
        assert_eq!(chain.count(), 4);
        assert_eq!(chain.size(), (8, 8));
        assert_eq!(chain.format(), crate::TextureFormat::Rgba8UnormSrgb);
        let expected = [4 * 8 * 8, 4 * 4 * 4, 4 * 2 * 2, 4];
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
        let chain = Mipmap::build((
            pixels.as_slice(),
            crate::TextureFormat::Rgba8UnormSrgb,
            (8u32, 8u32),
        ))
        .expect("prepare with tuple size");
        assert_eq!(chain.count(), 4);
        assert_eq!(chain.size(), (8, 8));

        // From<(&[u8], TextureFormat, [u32; 2])>
        let chain2 = Mipmap::build((
            pixels.as_slice(),
            crate::TextureFormat::Rgba8UnormSrgb,
            [8u32, 8u32],
        ))
        .expect("prepare with array size");
        assert_eq!(chain2.size(), (8, 8));

        // Explicit `TextureInput` struct literal — same transport as
        // `Renderer::create_texture` and `create_storage_texture`.
        let chain3 = Mipmap::build(crate::TextureInput {
            data: crate::TextureData::Bytes(pixels.clone()),
            options: crate::TextureOptions {
                size: Some(crate::Size::from([8u32, 8u32])),
                format: crate::TextureFormat::Rgba8UnormSrgb,
                ..Default::default()
            },
        })
        .expect("prepare with explicit TextureInput");
        assert_eq!(chain3.size(), (8, 8));
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
        let err = Mipmap::build((garbage.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
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
        let err = Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba32Uint))
            .expect_err("Rgba32Uint not supported by CPU mipmap path");
        match err {
            TextureError::UnsupportedMipmapFormat { format } => {
                assert_eq!(format, crate::TextureFormat::Rgba32Uint);
            }
            other => panic!("expected UnsupportedMipmapFormat, got {other:?}"),
        }

        // 3) Shape mismatch (raw path) → InvalidInput.
        let too_few_bytes = vec![0u8; 16];
        let err = Mipmap::build((
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
        let err = Mipmap::build((&[][..], crate::TextureFormat::Rgba8UnormSrgb, (0u32, 16u32)))
            .expect_err("zero-width raw input should fail");
        assert!(
            matches!(err, TextureError::InvalidInput(_)),
            "expected InvalidInput, got {err:?}"
        );
    }

    // Story: A prepared chain feeds back into create_texture and produces a
    // texture with the expected format + mip chain — the round-trip consumers
    // use for off-main-thread mip generation. Exercises both the
    // TextureInput::Prepared variant (Rust ergonomics) and the dedicated
    // create_texture_prepared cross-language entry point.
    // Story: raw pixels for a 3D texture keep one mip level, so the upload
    // writes every slice instead of resampling the first one.
    #[test]
    fn raw_3d_texture_keeps_one_level() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let pixels: Vec<u8> = (0..2 * 2 * 3 * 4).map(|value| value as u8).collect();
            let texture = renderer
                .create_texture((pixels.as_slice(), [2, 2, 3]))
                .await
                .expect("create_texture failed");
            let object = texture.object();
            assert_eq!(object.inner.dimension(), wgpu::TextureDimension::D3);
            assert_eq!(object.inner.mip_level_count(), 1);
        });
    }

    #[test]
    fn prepared_input_round_trips_through_create_texture() {
        pollster::block_on(async move {
            let pixels = vec![77u8; 8 * 8 * 4];
            let img =
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .unwrap();
            let chain = Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
                .expect("prepare ok");
            let level_count = chain.count() as u32;
            let r = crate::Renderer::new();
            let tex = r
                .create_texture(TextureData::Prepared(chain.clone()))
                .await
                .expect("create_texture from Prepared chain");
            assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba8UnormSrgb);
            assert_eq!(tex.object().inner.mip_level_count(), level_count);
            let sz = tex.size();
            assert_eq!(sz.width, 8);
            assert_eq!(sz.height, 8);

            // create_texture(chain) reaches the same path via
            // From<Mipmap> for TextureInput. No second method needed.
            let tex2 = r
                .create_texture(chain)
                .await
                .expect("create_texture from chain");
            assert_eq!(tex2.object().format, wgpu::TextureFormat::Rgba8UnormSrgb);
            assert_eq!(tex2.object().inner.mip_level_count(), level_count);
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
            assert_eq!(tex.object().format, wgpu::TextureFormat::R16Unorm);
            assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
        });
    }

    // Story: a format that needs a wgpu feature the device lacks is refused for
    // any usage, with the typed error naming the missing feature. Pure, no GPU.
    #[test]
    fn check_feature_names_the_missing_feature() {
        let empty = wgpu::Features::empty();
        let norm = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
        for format in [
            wgpu::TextureFormat::R16Unorm,
            wgpu::TextureFormat::Rg16Unorm,
            wgpu::TextureFormat::Rgba16Unorm,
        ] {
            match check_feature(empty, format) {
                Err(TextureError::UnsupportedFormatForUsage {
                    format: refused,
                    missing_feature,
                }) => {
                    assert_eq!(refused, format!("{format:?}"));
                    assert_eq!(missing_feature, "TEXTURE_FORMAT_16BIT_NORM");
                }
                other => panic!("expected UnsupportedFormatForUsage, got {other:?}"),
            }
            check_feature(norm, format).expect("with the feature");
        }
        for format in [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureFormat::Depth32Float,
        ] {
            check_feature(empty, format).expect("needs no feature");
        }
    }

    // Story: a usage the format can't take, such as storage on Depth32Float,
    // Rgba8UnormSrgb or Bgra8Unorm, and a depth format with layers are refused
    // with the error resize returns. Pure, against the WebGPU baseline.
    #[test]
    fn check_support_refuses_what_the_format_cannot_take() {
        let flat = wgpu::TextureDimension::D2;
        let deep = wgpu::TextureDimension::D3;
        let storage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
        let sampled = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let check = |format: wgpu::TextureFormat, usage, dimension| {
            let allowed = format
                .guaranteed_format_features(wgpu::Features::empty())
                .allowed_usages;
            check_support(allowed, format, usage, dimension)
        };
        for format in [
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::R8Unorm,
        ] {
            assert!(
                matches!(
                    check(format, storage, flat),
                    Err(TextureError::InvalidInput(_))
                ),
                "{format:?} takes no storage"
            );
            check(format, sampled, flat).expect("every format samples");
        }
        check(wgpu::TextureFormat::Rgba8Unorm, storage, flat).expect("Rgba8Unorm stores");
        check(wgpu::TextureFormat::Rgba8Unorm, sampled, deep).expect("a color format in 3D");
        assert!(matches!(
            check(wgpu::TextureFormat::Depth32Float, sampled, deep),
            Err(TextureError::InvalidInput(_))
        ));
    }
}
