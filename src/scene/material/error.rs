//! `MaterialError`: a value a Material setter can't use.

use crate::ShaderError;
use crate::scene::NotFinite;
use crate::texture::TextureError;

/// Errors from Material setters that take a value the material can't use.
/// A refused call changes nothing on the material.
#[derive(Debug, thiserror::Error)]
pub enum MaterialError {
    #[error("Material::texture_uv takes UV set 0 or 1, got {0}")]
    UvSet(u32),
    /// A number that isn't finite, such as NaN or an infinity, which `label` names with its call.
    #[error("{label} must be finite, got {value}")]
    NotFinite { label: String, value: f32 },
    /// A value the material's shader can't hold, such as a number past
    /// the range of an `f16` field that a `Material::custom` shader declares,
    /// or a texture for a slot that a `Material::custom` shader declares as a number.
    #[error(transparent)]
    Shader(#[from] ShaderError),
    /// A texture setter's input that makes no texture, such as options beside a `Texture`,
    /// which keeps the options it was created with, or an input that holds no data.
    #[error(transparent)]
    Texture(#[from] TextureError),
}

impl From<NotFinite> for MaterialError {
    fn from(NotFinite { label, value }: NotFinite) -> Self {
        MaterialError::NotFinite { label, value }
    }
}

/// A texture input takes the class the same input raises in `create_texture`,
/// and every other refusal is a `MaterialError`.
#[cfg(any(python, wasm, test))]
impl crate::errors::Classify for MaterialError {
    fn classify(&self) -> crate::errors::ErrorClass {
        match self {
            MaterialError::Texture(error) => error.classify(),
            _ => crate::errors::ErrorClass::Material,
        }
    }
}
