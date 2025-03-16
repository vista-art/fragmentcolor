#![cfg(feature = "python")]

use crate::{Pass, PassInput, PassObject, PassType, Shader};
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

    #[pyo3(name = "load_previous")]
    pub fn load_previous_py(&self) {
        *self.object.input.write() = PassInput::Load();
    }

    #[pyo3(name = "get_input")]
    pub fn get_input_py(&self) -> PassInput {
        self.object.get_input()
    }

    #[pyo3(name = "add_shader")]
    pub fn add_shader_py(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[pyo3(name = "set_clear_color")]
    pub fn set_clear_color_py(&self, color: [f32; 4]) {
        self.object.set_clear_color(color);
    }

    pub fn renderable_type(&self) -> &'static str {
        "Pass"
    }
}
