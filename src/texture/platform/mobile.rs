//! Mobile (Swift / Kotlin) uniffi bindings for `Texture` and `Mipmap`,
//! and the input types they take, `TextureInputMobile` and `TextureRegionMobile`.
//!
//! Mirrors the `wasm_bindgen` shim in `web.rs` and the `pyo3` shim in
//! `python.rs`: each foreign binding gets a thin wrapper around the
//! cross-platform `Texture` API. Mobile-specific methods carry a `_mobile`
//! suffix on the Rust side so the build-time doc scanner can keep them
//! separate from the Rust-only API, and every uniffi export carries an
//! explicit `name = "..."` attribute to expose an idiomatic camelCase form
//! in Swift and Kotlin.

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::error::platform::mobile::FragmentColorError;
use crate::{
    Mipmap, SamplerOptions, Size, Texture, TextureData, TextureFormat, TextureId, TextureRegion,
};

/// Uniffi mirror of [`TextureData`], which it converts into.
///
/// It leaves out `Empty`. Paths travel as `String`, handles as `Arc`.
///
/// The Rust core stays on `TextureData`. This enum exists only so the mobile
/// bindings can carry a concrete variant. Web and Python use their own dispatch
/// (`TryFrom<JsValue>` and Python kwargs) and never see this type.
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

/// Mobile-only flat version of [`TextureRegion`] for uniffi FFI.
///
/// Uniffi cannot marshal fixed-size arrays (`[u32; 3]`), so the mobile
/// binding exposes the 3D origin and size as individual `u32` fields.
/// Each size is at least 1, and a write without a region covers the whole texture.
///
/// Swift and Kotlin see it as `TextureRegion`, the name every other binding uses.
#[derive(Debug, Clone, uniffi::Record)]
#[uniffi(name = "TextureRegion")]
pub struct TextureRegionMobile {
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
    /// Width of the region to write, at least 1.
    pub size_width: u32,
    /// Height of the region to write, at least 1.
    pub size_height: u32,
    /// Depth of the region to write, at least 1.
    pub size_depth: u32,
    pub bytes_per_row: Option<u32>,
    pub rows_per_image: Option<u32>,
}

impl From<TextureRegionMobile> for TextureRegion {
    fn from(m: TextureRegionMobile) -> Self {
        TextureRegion {
            origin: [m.origin_x, m.origin_y, m.origin_z],
            size: [m.size_width, m.size_height, m.size_depth],
            bytes_per_row: m.bytes_per_row,
            rows_per_image: m.rows_per_image,
        }
    }
}

#[uniffi::export]
impl Texture {
    /// Return the stable [`TextureId`] for this texture instance. The id is
    /// valid within the `Renderer` that created it. Mobile callers receive
    /// a copy (uniffi::Record) so no lifetime plumbing is needed.
    #[uniffi::method(name = "id")]
    #[lsp_doc("docs/api/texture/texture/id.md")]
    pub fn id_mobile(&self) -> TextureId {
        *self.id()
    }

    /// Return the texture size (w × h[× d]). Mirrors the canonical
    /// `Texture::size()` — see [`crate::Size`].
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/texture/texture/size.md")]
    pub fn size_mobile(&self) -> Size {
        self.size()
    }

    /// Return the aspect ratio (width / height) as an `f32`.
    #[uniffi::method(name = "aspect")]
    #[lsp_doc("docs/api/texture/texture/aspect.md")]
    pub fn aspect_mobile(&self) -> f32 {
        self.aspect()
    }

    /// Sets the sampler options: filtering, wrapping and an optional depth compare.
    /// Swift calls it as `setSamplerOptions(opts)` through its extension, and Kotlin as is.
    #[uniffi::method(name = "setSamplerOptions")]
    #[lsp_doc("docs/api/texture/texture/set_sampler_options.md")]
    pub fn set_sampler_options_mobile(&self, opts: SamplerOptions) {
        self.set_sampler_options(opts);
    }

    /// Upload raw pixel data into the whole texture. `bytes` must be tightly
    /// packed for the texture's format; see `Texture::write` for supported
    /// formats and alignment rules. Mirrors `Texture::write(&[u8])`.
    #[uniffi::method(name = "write")]
    #[lsp_doc("docs/api/texture/texture/write.md")]
    pub fn write_mobile(&self, bytes: Vec<u8>) -> Result<(), FragmentColorError> {
        Ok(self.write(&bytes)?)
    }

    /// Upload raw pixel data into a sub-region of the texture, each side at least 1.
    /// Call `write()` for the whole texture. Mirrors `Texture::write_region`.
    #[uniffi::method(name = "writeRegion")]
    #[lsp_doc("docs/api/texture/texture/write_region.md")]
    pub fn write_region_mobile(
        &self,
        bytes: Vec<u8>,
        region: TextureRegionMobile,
    ) -> Result<(), FragmentColorError> {
        Ok(self.write_region(&bytes, region)?)
    }

    /// Recreates the texture at `size`. Its pixels start blank.
    #[uniffi::method(name = "resize")]
    #[lsp_doc("docs/api/texture/texture/resize.md")]
    pub fn resize_mobile(&self, size: Size) -> Result<(), FragmentColorError> {
        Ok(self.resize(size)?)
    }

    /// Read back the mip-0 contents of this texture as tightly-packed bytes
    /// in the texture's native format. Uniffi exposes this as a Swift
    /// `suspend fun` / Kotlin `suspend fun` automatically. Foreign callers
    /// await this in a coroutine or `Task`; the underlying GPU readback is
    /// driven by the async `texture::read::read_pixels` path.
    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/texture/texture/get_image.md")]
    pub async fn get_image_mobile(self: Arc<Self>) -> Result<Vec<u8>, FragmentColorError> {
        Ok(self.get_image().await?)
    }
}

#[uniffi::export]
impl Mipmap {
    /// Build a chain from raw bytes + format (+ optional size). If
    /// `size` is `None`, `bytes` is decoded as an image (PNG / JPEG / etc.);
    /// if `Some(size)`, `bytes` is treated as raw pixel data already laid
    /// out for `format` at `size`. Pure CPU work — call from a Swift `Task`
    /// or Kotlin coroutine on a background dispatcher, then hand the chain
    /// to `renderer.createTexture(chain:)` for the GPU upload.
    ///
    /// Mobile shim takes the fields directly because uniffi can't marshal
    /// `impl Into<TextureInput>`. `size` defaults to `None`, and the Swift
    /// extensions add unlabeled `Mipmap.build(bytes, format)` (encoded) and
    /// `Mipmap.build(bytes, format, size)` (raw) overloads on top.
    #[uniffi::constructor(name = "build", default(size = None))]
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build_mobile(
        bytes: Vec<u8>,
        format: TextureFormat,
        size: Option<Size>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        let input = crate::TextureInput {
            data: crate::TextureData::Bytes(bytes),
            options: crate::TextureOptions {
                size,
                format,
                ..Default::default()
            },
        };
        Ok(Arc::new(Self::build(input)?))
    }

    #[uniffi::method(name = "format")]
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format_mobile(&self) -> TextureFormat {
        TextureFormat::from_wgpu(self.format)
    }

    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size_mobile(&self) -> Size {
        self.size()
    }

    #[uniffi::method(name = "count")]
    #[lsp_doc("docs/api/texture/mipmap/count.md")]
    pub fn count_mobile(&self) -> u32 {
        self.count() as u32
    }

    /// Every mip level's bytes, the base level first.
    #[uniffi::method(name = "levels")]
    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn levels_mobile(&self) -> Vec<Vec<u8>> {
        self.levels().to_vec()
    }

    /// Return the bytes for a single mip level. Use `count()` to discover
    /// the valid range.
    #[uniffi::method(name = "level")]
    #[lsp_doc("docs/api/texture/mipmap/level.md")]
    pub fn level_mobile(&self, index: u32) -> Result<Vec<u8>, FragmentColorError> {
        Ok(self.level(index as usize)?.to_vec())
    }
}
