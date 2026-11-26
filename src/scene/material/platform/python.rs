#![cfg(python)]

use pyo3::prelude::*;

use crate::{Material, MaterialError, PbrMaterial, Shader};

/// A material from Python, or one handed back: the material object itself,
/// since Python has no `Material` class.
#[derive(FromPyObject, IntoPyObject)]
pub enum PyMaterial {
    Pbr(PbrMaterial),
    Custom(Shader),
}

impl From<PyMaterial> for Material {
    fn from(material: PyMaterial) -> Self {
        match material {
            PyMaterial::Pbr(material) => material.into(),
            PyMaterial::Custom(shader) => shader.into(),
        }
    }
}

impl From<&Material> for PyMaterial {
    fn from(material: &Material) -> Self {
        match material {
            Material::Pbr(material) => Self::Pbr(material.clone()),
            Material::Custom(shader) => Self::Custom(shader.clone()),
        }
    }
}

/// A value the material can't use raises `MaterialError`,
/// a `FragmentColorError` that is also a `ValueError`.
impl From<MaterialError> for PyErr {
    fn from(error: MaterialError) -> Self {
        crate::errors::raise_error(&error)
    }
}
