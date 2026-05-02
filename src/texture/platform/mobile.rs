//! Mobile (Swift / Kotlin) uniffi bindings for `Texture`.
//!
//! Mirrors the `wasm_bindgen` shim in `web.rs` and the `pyo3` shim in
//! `python.rs`: each foreign binding gets a thin wrapper around the
//! cross-platform `Texture` API. The Rust core method on `Texture` is
//! `set_sampler_options(opts: SamplerOptions)`; the mobile shim is named
//! `set_sampler_options_mobile` so the build-time doc scanner can keep
//! it separate from the Rust-only API, with the uniffi `name = "..."`
//! attribute exposing the idiomatic camelCase form.

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::renderer::platform::mobile::FragmentColorError;
use crate::{SamplerOptions, Size, Texture, TextureFormat, TextureMipChain};

#[uniffi::export]
impl Texture {
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
