//! `MaterialError`: a value a Material setter can't use.

use crate::ShaderError;
use crate::scene::NotFinite;

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
    /// the range of an `f16` field that a `Material::custom` shader declares.
    #[error(transparent)]
    Shader(#[from] ShaderError),
}

impl From<NotFinite> for MaterialError {
    fn from(NotFinite { label, value }: NotFinite) -> Self {
        MaterialError::NotFinite { label, value }
    }
}

#[cfg(any(python, wasm, test))]
impl crate::errors::Classify for MaterialError {
    fn classify(&self) -> crate::errors::ErrorClass {
        crate::errors::ErrorClass::Material
    }
}
