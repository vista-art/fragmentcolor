#![cfg(feature = "python")]

use crate::{Pass, PassObject, PassType, Shader};
use pyo3::prelude::*;
use std::sync::Arc;

#[pymethods]
impl Pass {
    #[new]
    pub fn new_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Render)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "compute")]
    pub fn compute_py(name: &str) -> Self {
        Self {
            object: Arc::new(PassObject::new(name, PassType::Compute)),
        }
    }

    #[staticmethod]
    #[pyo3(name = "from_shader")]
    pub fn from_shader_py(name: &str, shader: &Shader) -> Self {
        Self {
            object: Arc::new(PassObject::from_shader_object(name, shader.object.clone())),
        }
    }

    #[pyo3(name = "passes")]
    pub fn passes_py(&self) -> crate::PyPassIterator {
        crate::PyPassIterator(vec![self.object.clone()])
    }

    pub fn renderable_type(&self) -> &'static str {
        "Frame"
    }
}
