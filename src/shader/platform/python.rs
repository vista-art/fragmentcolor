#![cfg(python)]

use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;

#[pymethods]
impl Shader {
    #[new]
    pub fn new_py(source: &str) -> Result<Self, ShaderError> {
        Shader::new(source)
    }

    #[pyo3(name = "set")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> Result<(), PyErr> {
        Python::attach(|py| -> Result<(), PyErr> {
            // If it's a Texture object, map to UniformData::Texture with id only
            if let Ok(tex) = value.bind(py).downcast::<crate::texture::Texture>() {
                let meta = crate::texture::TextureMeta::with_id_only(tex.borrow().id.clone());
                return self
                    .object
                    .set(key, UniformData::Texture(meta))
                    .map_err(|e| e.into());
            }
            // Fallback: try to extract as UniformData via derived conversions
            let ud: UniformData = value.bind(py).extract()?;
            self.object.set(key, ud).map_err(|e| e.into())
        })
    }

    #[pyo3(name = "get")]
    pub fn get_py(&self, key: &str) -> Result<Py<PyAny>, PyErr> {
        Python::attach(|py| -> Result<Py<PyAny>, PyErr> {
            let data = self.object.get_uniform_data(key)?;

            let object = data.into_pyobject(py)?;

            Ok(object.unbind())
        })
    }

    #[pyo3(name = "list_uniforms")]
    pub fn list_uniforms_py(&self) -> Vec<String> {
        self.list_uniforms()
    }

    #[pyo3(name = "list_keys")]
    pub fn list_keys_py(&self) -> Vec<String> {
        self.list_keys()
    }

    pub fn renderable_type(&self) -> &'static str {
        "Shader"
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.pass.clone()])
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    pub fn default_py() -> Self {
        Shader::default()
    }
}
