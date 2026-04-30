//! Placeholder for cross-platform external texture creation.
//!
//! Wraps a native platform video-frame source into a wgpu external-texture
//! sampler so shaders can read directly from decoded video without an
//! intermediate CPU upload. The Web binding accepts an `HtmlVideoElement`;
//! iOS will accept a `CVPixelBuffer` pointer and Android a `SurfaceTexture`
//! handle once the per-platform plumbing is in place. Today every entry
//! point returns `RendererError::Error("not implemented yet")` — the API
//! exists on every binding so users can write portable code paths against
//! it now and the underlying implementation can fill in incrementally.

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct ExternalTextureHandle {
    #[cfg(wasm)]
    pub(crate) _inner: wgpu::ExternalTexture,
}

#[cfg(wasm)]
pub fn create_external_texture(
    _r: &crate::renderer::Renderer,
    _video: &web_sys::HtmlVideoElement,
) -> Result<ExternalTextureHandle, crate::renderer::error::RendererError> {
    Err(crate::renderer::error::RendererError::Error(
        "External texture import not implemented yet".into(),
    ))
}

#[cfg(mobile)]
pub fn create_external_texture_from_native(
    _r: &crate::renderer::Renderer,
    _source_ptr: u64,
) -> Result<std::sync::Arc<ExternalTextureHandle>, crate::renderer::error::RendererError> {
    Err(crate::renderer::error::RendererError::Error(
        "External texture import not implemented yet".into(),
    ))
}
