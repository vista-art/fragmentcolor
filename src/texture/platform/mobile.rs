//! Mobile (Swift / Kotlin) uniffi bindings for `Texture`.
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

use crate::renderer::platform::mobile::FragmentColorError;
use crate::texture::TextureRegionMobile;
use crate::{SamplerOptions, Size, Texture, TextureFormat, TextureId, TextureMipChain};

#[uniffi::export]
impl Texture {
    /// Return the stable [`TextureId`] for this texture instance. The id is
    /// valid within the `Renderer` that created it. Mobile callers receive
    /// a copy (uniffi::Record) so no lifetime plumbing is needed.
    #[uniffi::method(name = "id")]
    #[lsp_doc("docs/api/core/texture/id.md")]
    pub fn id_mobile(&self) -> TextureId {
        *self.id()
    }

    /// Return the texture size (w × h[× d]). Mirrors the canonical
    /// `Texture::size()` — see [`crate::Size`].
    #[uniffi::method(name = "size")]
    #[lsp_doc("docs/api/core/texture/size.md")]
    pub fn size_mobile(&self) -> Size {
        self.size()
    }

    /// Return the aspect ratio (width / height) as an `f32`.
    #[uniffi::method(name = "aspect")]
    #[lsp_doc("docs/api/core/texture/aspect.md")]
    pub fn aspect_mobile(&self) -> f32 {
        self.aspect()
    }

    /// Update the texture sampler options (filtering, wrapping, optional
    /// depth-compare). Mirrors the Web `setSamplerOptions` and Python
    /// `set_sampler_options` entry points; foreign bindings see this
    /// method as `setSamplerOptions(opts:)` (Swift) /
    /// `setSamplerOptions(opts)` (Kotlin) once the extension shims map
    /// the camelCase form back onto a single overload.
    #[uniffi::method(name = "setSamplerOptions")]
    #[lsp_doc("docs/api/core/texture/set_sampler_options.md")]
    pub fn set_sampler_options_mobile(&self, opts: SamplerOptions) {
        self.set_sampler_options(opts);
    }

    /// Upload raw pixel data into the whole texture. `bytes` must be tightly
    /// packed for the texture's format; see `Texture::write` for supported
    /// formats and alignment rules. Mirrors `Texture::write(&[u8])`.
    #[uniffi::method(name = "write")]
    #[lsp_doc("docs/api/core/texture/write.md")]
    pub fn write_mobile(&self, bytes: Vec<u8>) -> Result<(), FragmentColorError> {
        self.write(&bytes).map_err(FragmentColorError::from)
    }

    /// Upload raw pixel data into a sub-region of the texture. Pass a
    /// `TextureRegionMobile` with all-zero `size_*` fields for a whole-texture
    /// write (equivalent to `write()`). Mirrors `Texture::write_region`.
    #[uniffi::method(name = "writeRegion")]
    #[lsp_doc("docs/api/core/texture/write_region.md")]
    pub fn write_region_mobile(
        &self,
        bytes: Vec<u8>,
        region: TextureRegionMobile,
    ) -> Result<(), FragmentColorError> {
        self.write_region(&bytes, region)
            .map_err(FragmentColorError::from)
    }

    /// Read back the mip-0 contents of this texture as tightly-packed bytes
    /// in the texture's native format. Uniffi exposes this as a Swift
    /// `suspend fun` / Kotlin `suspend fun` automatically. Foreign callers
    /// await this in a coroutine or `Task`; the underlying GPU readback is
    /// driven by the async `texture::read::read_pixels` path.
    #[uniffi::method(name = "getImage")]
    #[lsp_doc("docs/api/core/texture/get_image.md")]
    pub async fn get_image_mobile(self: Arc<Self>) -> Result<Vec<u8>, FragmentColorError> {
        self.get_image().await.map_err(FragmentColorError::from)
    }
}

#[uniffi::export]
impl TextureMipChain {
    /// Build a chain from raw bytes + format (+ optional size). If
    /// `size` is `None`, `bytes` is decoded as an image (PNG / JPEG / etc.);
    /// if `Some(size)`, `bytes` is treated as raw pixel data already laid
    /// out for `format` at `size`. Pure CPU work — call from a Swift `Task`
    /// or Kotlin coroutine on a background dispatcher, then hand the chain
    /// to `renderer.createTexture(chain:)` for the GPU upload.
    ///
    /// Mobile shim takes the fields directly because uniffi can't marshal
    /// `impl Into<TextureInput>`. Swift / Kotlin extensions wrap this
    /// constructor so end users call `TextureMipChain.prepare(bytes:format:)`
    /// (encoded) or `TextureMipChain.prepare(bytes:format:size:)` (raw)
    /// without seeing the underlying `TextureInput` plumbing.
    #[uniffi::constructor(name = "prepare")]
    #[lsp_doc("docs/api/core/texture_mip_chain/prepare.md")]
    pub fn prepare_mobile(
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
        Self::prepare(input)
            .map(Arc::new)
            .map_err(FragmentColorError::from)
    }

    #[uniffi::method(name = "format")]
    #[lsp_doc("docs/api/core/texture_mip_chain/format.md")]
    pub fn format_mobile(&self) -> TextureFormat {
        self.format.into()
    }

    #[uniffi::method(name = "baseSize")]
    #[lsp_doc("docs/api/core/texture_mip_chain/base_size.md")]
    pub fn base_size_mobile(&self) -> Size {
        let (w, h) = self.base_size();
        Size::from([w, h])
    }

    #[uniffi::method(name = "levelCount")]
    #[lsp_doc("docs/api/core/texture_mip_chain/level_count.md")]
    pub fn level_count_mobile(&self) -> u32 {
        self.level_count() as u32
    }

    /// Return the bytes for a single mip level. Use `levelCount()` to discover
    /// the valid range.
    #[uniffi::method(name = "level")]
    #[lsp_doc("docs/api/core/texture_mip_chain/levels.md")]
    pub fn level_mobile(&self, index: u32) -> Result<Vec<u8>, FragmentColorError> {
        let levels = self.levels();
        let idx = index as usize;
        if idx >= levels.len() {
            return Err(FragmentColorError::Render(format!(
                "level {} out of range; chain has {} levels",
                idx,
                levels.len()
            )));
        }
        Ok(levels[idx].clone())
    }
}
