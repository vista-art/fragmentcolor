#![cfg(mobile)]

use std::sync::Arc;

use crate::errors::platform::mobile::FragmentColorError;
use crate::{Material, MaterialError, PbrMaterial, Shader, ToonMaterial, UnlitMaterial};

/// What a `Model` draws with, as Swift and Kotlin see it: an enum that holds the
/// material object, which is how both languages spell a value of one of four types.
/// Each case names its object, `material`, or `shader` for a custom one, so Kotlin
/// reads `Material.Pbr.material` where uniffi would name a tuple field `v1`.
/// Uniffi marshals objects as `Arc`s, which the Rust `Material` holds by value,
/// so this mirror carries the same cases under the same name.
#[derive(Debug, Clone, uniffi::Enum)]
#[uniffi(name = "Material")]
pub enum MobileMaterial {
    Pbr { material: Arc<PbrMaterial> },
    Unlit { material: Arc<UnlitMaterial> },
    Toon { material: Arc<ToonMaterial> },
    Custom { shader: Arc<Shader> },
}

impl From<MobileMaterial> for Material {
    fn from(material: MobileMaterial) -> Self {
        match material {
            MobileMaterial::Pbr { material } => (*material).clone().into(),
            MobileMaterial::Unlit { material } => (*material).clone().into(),
            MobileMaterial::Toon { material } => (*material).clone().into(),
            MobileMaterial::Custom { shader } => (*shader).clone().into(),
        }
    }
}

impl From<&Material> for MobileMaterial {
    fn from(material: &Material) -> Self {
        match material {
            Material::Pbr(material) => Self::Pbr {
                material: Arc::new(material.clone()),
            },
            Material::Unlit(material) => Self::Unlit {
                material: Arc::new(material.clone()),
            },
            Material::Toon(material) => Self::Toon {
                material: Arc::new(material.clone()),
            },
            Material::Custom(shader) => Self::Custom {
                shader: Arc::new(shader.clone()),
            },
        }
    }
}

/// A value the material can't use reaches Swift and Kotlin as a render error,
/// and a texture input as the error `createTexture` throws for it.
impl From<MaterialError> for FragmentColorError {
    fn from(error: MaterialError) -> Self {
        match error {
            MaterialError::Texture(error) => error.into(),
            error => FragmentColorError::Render(error.to_string()),
        }
    }
}
