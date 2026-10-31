//! `MaterialError`: a value a Material setter can't use.

/// Errors from Material setters that take a value the material can't use.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MaterialError {
    #[error("Material::texture_uv takes UV set 0 or 1, got {0}")]
    UvSet(u32),
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for MaterialError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::Material
    }
}
