use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Context not initialized")]
    NoContext,
    /// The input bytes could not be decoded as an image. Distinct from
    /// [`TextureError::UnsupportedMipmapFormat`] (which means the bytes were
    /// fine but the requested target format isn't supported for CPU mipmap
    /// generation) and [`TextureError::InvalidInput`] (which means the bytes
    /// parsed but the declared shape was wrong). Surfaces failures from
    /// `image::load_from_memory` / `image::open` for callers that want to
    /// log "this tile is corrupt" specifically.
    #[error("Image decode failed: {0}")]
    MalformedImageError(#[from] image::ImageError),
    /// The requested target format isn't supported by the CPU mipmap
    /// dispatcher. Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`,
    /// `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`,
    /// `Rg16Unorm`, `Rgba16Unorm`. For other formats, ship a pre-baked KTX2
    /// chain (`TextureInput::Ktx2*`) or set `TextureOptions { mipmaps: false, .. }`.
    #[error(
        "Unsupported format for CPU mipmap generation: {format:?}. Supported formats: Rgba8/Bgra8 (Unorm + Srgb), R8Unorm, Rg8Unorm, R16Unorm, Rg16Unorm, Rgba16Unorm. Use TextureInput::Ktx2* for other formats, or set mipmaps=false."
    )]
    UnsupportedMipmapFormat { format: crate::TextureFormat },
    /// The provided buffer didn't match the declared shape: zero size, byte
    /// count too small for `bpp * width * height`, or similar. Distinct from
    /// [`TextureError::MalformedImageError`] (which means the bytes were
    /// invalid as an image) — `InvalidInput` means the bytes were a valid
    /// blob but didn't match what you said they were.
    #[error("Invalid texture input: {0}")]
    InvalidInput(String),
    /// The active device doesn't have the wgpu feature required to use the
    /// requested format with the requested usage. Common case: `R16Unorm`
    /// (and its Rg/Rgba/Snorm cousins) need
    /// [`wgpu::Features::TEXTURE_FORMAT_16BIT_NORM`] for `TEXTURE_BINDING`.
    /// FragmentColor requests every feature the adapter advertises at
    /// device creation (see `request_device` in `renderer/platform/all.rs`),
    /// so the only path to this error is a device that genuinely doesn't
    /// advertise the format-feature this format needs. Surfaces capability constraints at the API boundary
    /// instead of letting them detonate as runtime validation cascades on
    /// first use. The `format` field carries the underlying
    /// `wgpu::TextureFormat` so the message stays accurate for variants
    /// (e.g. `R16Snorm`) that don't have a corresponding `crate::TextureFormat`.
    #[error(
        "Texture format {format:?} is not supported by the active device for the requested usage (missing wgpu feature {missing_feature:?}). The adapter does not advertise this feature; pick a different format or run on a device that supports it."
    )]
    UnsupportedFormatForUsage {
        format: wgpu::TextureFormat,
        missing_feature: wgpu::Features,
    },
    #[error("Failed to create texture: {0}")]
    CreateTextureError(String),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::shader::error::ShaderError),
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    #[error("Renderer error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
use pyo3::exceptions::PyException as PyFragmentColorError;

#[cfg(python)]
impl From<pyo3::PyErr> for TextureError {
    fn from(e: pyo3::PyErr) -> Self {
        TextureError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<TextureError> for pyo3::PyErr {
    fn from(e: TextureError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for TextureError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        TextureError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<TextureError> for wasm_bindgen::JsValue {
    fn from(error: TextureError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: TextureError wraps failures from creation paths and shader conversion.
    #[test]
    fn texture_error_variants_and_from_shader() {
        let e1 = TextureError::NoContext;
        assert!(e1.to_string().contains("Context not initialized"));

        let e2 = TextureError::CreateTextureError("bad".into());
        assert!(e2.to_string().contains("Failed to create texture"));

        // From ShaderError
        let se = crate::shader::error::ShaderError::UniformNotFound("u".into());
        let e3: TextureError = se.into();
        assert!(matches!(e3, TextureError::ShaderError(_)));
    }
}
