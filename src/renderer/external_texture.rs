#[cfg(wasm)]
pub struct ExternalTextureHandle {
    pub(crate) inner: wgpu::ExternalTexture,
}

#[cfg(wasm)]
pub fn create_from_html_video(
    _r: &crate::renderer::Renderer,
    _video: &web_sys::HtmlVideoElement,
) -> Result<ExternalTextureHandle, crate::renderer::error::RendererError> {
    // Minimal stub: actual import requires browser support and wgpu-web plumbing.
    Err(crate::renderer::error::RendererError::Error(
        "External texture import not implemented yet".into(),
    ))
}
