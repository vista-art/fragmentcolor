#![cfg(mobile)]

use std::sync::Arc;

use crate::{Material, PbrMaterial, Shader, ToonMaterial, UnlitMaterial};

/// What a `Model` draws with, as the uniffi plumbing carries it: one case per
/// material type, holding the object. Uniffi can't marshal `impl Into<Material>` or
/// return one of several object types, so the raw `Model(mesh:material:)` constructor
/// and `materialHandle()` take and return this enum. The Swift and Kotlin extensions
/// take and return the material object itself, as JavaScript and Python do, and no
/// doc shows the handle.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum MaterialHandle {
    Pbr(Arc<PbrMaterial>),
    Unlit(Arc<UnlitMaterial>),
    Toon(Arc<ToonMaterial>),
    Custom(Arc<Shader>),
}

impl MaterialHandle {
    /// The material the handle holds. A method rather than a `From` impl, so the
    /// transpilers don't read the handle as the Swift and Kotlin spelling of `Material`.
    pub(crate) fn into_material(self) -> Material {
        match self {
            Self::Pbr(material) => (*material).clone().into(),
            Self::Unlit(material) => (*material).clone().into(),
            Self::Toon(material) => (*material).clone().into(),
            Self::Custom(shader) => (*shader).clone().into(),
        }
    }
}

impl From<&Material> for MaterialHandle {
    fn from(material: &Material) -> Self {
        match material {
            Material::Pbr(material) => Self::Pbr(Arc::new(material.clone())),
            Material::Unlit(material) => Self::Unlit(Arc::new(material.clone())),
            Material::Toon(material) => Self::Toon(Arc::new(material.clone())),
            Material::Custom(shader) => Self::Custom(Arc::new(shader.clone())),
        }
    }
}
