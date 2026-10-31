use std::convert::Infallible;

use thiserror::Error;

use crate::ScreenRegionError;

#[derive(Error, Debug)]
pub enum TextureError {
    /// A call that needs a GPU context, such as a resize of a target with no surface yet.
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
    MalformedImageError(String),
    /// The requested target format isn't supported by the CPU mipmap
    /// dispatcher. Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`,
    /// `Bgra8Unorm`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`,
    /// `Rgba16Unorm`, `Rgba16Float`, `Rgba32Float`, and the aliases for them. For other formats, ship a pre-baked KTX2
    /// chain (`TextureInput::Ktx2*`) or set `TextureOptions { mipmaps: false, .. }`.
    #[error(
        "Unsupported format for CPU mipmap generation: {format:?}. Supported formats: Rgba8/Bgra8 (Unorm + Srgb), R8Unorm, Rg8Unorm, R16Unorm, Rg16Unorm, Rgba16Unorm, Rgba16Float, Rgba32Float. Use TextureInput::Ktx2* for other formats, or set mipmaps=false."
    )]
    UnsupportedMipmapFormat { format: crate::TextureFormat },
    /// The provided buffer didn't match the declared shape: zero size, byte
    /// count too small for `bpp * width * height`, or similar. Distinct from
    /// [`TextureError::MalformedImageError`] (which means the bytes were
    /// invalid as an image) — `InvalidInput` means the bytes were a valid
    /// blob but didn't match what you said they were.
    #[error("Invalid texture input: {0}")]
    InvalidInput(String),

    /// A `ScreenRegion` given as a write region whose bounds don't make a region.
    #[error(transparent)]
    Region(#[from] ScreenRegionError),
    /// A texture whose pixels can't be read back, such as a window target's.
    #[error("Texture readback failed: {0}")]
    Readback(String),
    /// Only a texture with `RENDER_ATTACHMENT` usage, one a pass can draw into, resizes.
    #[error("Only a texture a pass can draw into resizes. Create it with RENDER_ATTACHMENT usage.")]
    FixedSize,
    /// A format the device can't use without a wgpu feature it lacks, such as
    /// `R16Unorm` without `TEXTURE_FORMAT_16BIT_NORM`. `format` names the format,
    /// which may have no `TextureFormat`, and `missing_feature` names the features it lacks.
    #[error(
        "Texture format {format} is not supported by the active device for the requested usage (missing wgpu feature {missing_feature}). The adapter does not advertise this feature; pick a different format or run on a device that supports it."
    )]
    UnsupportedFormatForUsage {
        format: String,
        missing_feature: String,
    },
    /// A source that can't build a texture, such as an empty Mipmap or an unreadable KTX2 file.
    #[error("Failed to create texture: {0}")]
    CreateTextureError(String),
    /// A `ShaderError` a texture call passes on.
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::ShaderError),
    /// A bind group layout that can't be built.
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    /// A GPU error an earlier frame left, which a readback reports before it reads.
    #[error("Device error ({kind}) in '{label}': {detail}")]
    DeviceError {
        kind: crate::ErrorKind,
        label: String,
        detail: String,
    },
    /// Any other texture failure.
    #[error("Renderer error: {0}")]
    Error(String),
}

impl TextureError {
    /// This error with `name` before its text, so a failed upload says which source it read.
    pub(crate) fn name(self, name: &str) -> Self {
        match self {
            Self::MalformedImageError(text) => Self::MalformedImageError(format!("{name}: {text}")),
            Self::InvalidInput(text) => Self::InvalidInput(format!("{name}: {text}")),
            Self::CreateTextureError(text) => Self::CreateTextureError(format!("{name}: {text}")),
            other => other,
        }
    }
}

/// A region form that always converts, such as `[x, y, w, h]`, lifts with `?`.
impl From<Infallible> for TextureError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for TextureError {
    fn from(e: pyo3::PyErr) -> Self {
        TextureError::Error(e.to_string())
    }
}

/// Four texture errors take their own class of `FragmentColorError`, so a caller can tell them apart.
#[cfg(any(python, wasm, test))]
impl crate::error::Classify for TextureError {
    fn classify(&self) -> crate::error::ErrorClass {
        use crate::error::ErrorClass;
        match self {
            TextureError::InvalidInput(_) | TextureError::Region(_) => ErrorClass::InvalidInput,
            TextureError::UnsupportedMipmapFormat { .. } => ErrorClass::UnsupportedMipmapFormat,
            TextureError::MalformedImageError(_) => ErrorClass::MalformedImage,
            TextureError::DeviceError { .. } => ErrorClass::Device,
            _ => ErrorClass::Base,
        }
    }
}

#[cfg(python)]
impl From<TextureError> for pyo3::PyErr {
    fn from(e: TextureError) -> Self {
        let message = e.to_string();
        match e {
            TextureError::DeviceError {
                kind,
                label,
                detail,
            } => crate::error::DeviceError::raise(kind, label, detail, message),
            other => crate::error::raise_error(&other),
        }
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

/// The JS `Error` a texture error throws, named after its class.
/// A device error also carries its `kind`, `label` and `detail`.
#[cfg(wasm)]
impl From<TextureError> for crate::error::WebError {
    fn from(error: TextureError) -> Self {
        let message = error.to_string();
        match error {
            TextureError::DeviceError {
                kind,
                label,
                detail,
            } => crate::core::throw_device(kind, &label, &detail, &message),
            other => crate::error::WebError::from_error(&other),
        }
    }
}

#[cfg(wasm)]
impl From<TextureError> for wasm_bindgen::JsValue {
    fn from(error: TextureError) -> Self {
        crate::error::WebError::from(error).into()
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
        let se = crate::ShaderError::UniformNotFound("u".into());
        let e3: TextureError = se.into();
        assert!(matches!(e3, TextureError::ShaderError(_)));
    }
}
