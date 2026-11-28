#![cfg(python)]

use pyo3::prelude::*;

use crate::bindings::python::refuse;
use crate::{Material, MaterialError, PbrMaterial, Shader, ToonMaterial, UnlitMaterial};

/// A material from Python, or one handed back: the material object itself,
/// since Python has no `Material` class. `Other` keeps any other value a caller passes,
/// and no material handed back is one.
#[derive(FromPyObject, IntoPyObject)]
pub enum PyMaterial {
    Pbr(PbrMaterial),
    Unlit(UnlitMaterial),
    Toon(ToonMaterial),
    Custom(Shader),
    Other(Py<PyAny>),
}

impl PyMaterial {
    /// The material, or a `TypeError` that names `label` for a value that is no material,
    /// as JavaScript throws one.
    pub(crate) fn read(self, label: &str) -> PyResult<Material> {
        match self {
            PyMaterial::Pbr(material) => Ok(material.into()),
            PyMaterial::Unlit(material) => Ok(material.into()),
            PyMaterial::Toon(material) => Ok(material.into()),
            PyMaterial::Custom(shader) => Ok(shader.into()),
            PyMaterial::Other(value) => Python::attach(|py| {
                let forms = "expected a PbrMaterial, an UnlitMaterial, a ToonMaterial or a Shader";
                Err(refuse(value.bind(py), format_args!("{label}: {forms}")))
            }),
        }
    }
}

impl From<&Material> for PyMaterial {
    fn from(material: &Material) -> Self {
        match material {
            Material::Pbr(material) => Self::Pbr(material.clone()),
            Material::Unlit(material) => Self::Unlit(material.clone()),
            Material::Toon(material) => Self::Toon(material.clone()),
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
