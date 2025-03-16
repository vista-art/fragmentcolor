#![cfg(feature = "python")]

use crate::{Shader, ShaderError, UniformData};
use pyo3::prelude::*;

#[pymethods]
impl Shader {
    #[new]
    pub fn new_py(source: &str) -> Result<Self, ShaderError> {
        Shader::new(source)
    }

    #[pyo3(name = "set")]
    pub fn set_py(&self, key: &str, value: UniformData) -> Result<(), PyErr> {
        self.object.set(key, value).map_err(|e| e.into())
    }

    #[pyo3(name = "get")]
    pub fn get_py(&self, key: &str) -> Result<PyObject, PyErr> {
        Python::with_gil(|py| -> Result<PyObject, PyErr> {
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
}
