#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::bindings::python::{PyChannels, PyFloat};
use crate::{AlphaMode, MaterialMap, Shader, UnlitMaterial};

#[pymethods]
impl UnlitMaterial {
    #[new]
    #[lsp_doc("docs/api/scene/unlit_material/new.md")]
    pub fn new_py() -> Self {
        UnlitMaterial::new()
    }

    #[pyo3(name = "shader")]
    #[lsp_doc("docs/api/scene/unlit_material/shader.md")]
    pub fn shader_py(&self) -> Shader {
        // A clone of the inner Shader is an Arc-share, so the caller works on the
        // material's own shader, as Rust's `&Shader` does.
        self.shader.clone()
    }

    #[pyo3(name = "base_color")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color.md")]
    pub fn base_color_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.base_color(color.read_array("UnlitMaterial.base_color")?)?)
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.alpha_cutoff(value.read_finite("UnlitMaterial.alpha_cutoff: value")?)?)
    }

    #[pyo3(name = "alpha_mode")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_mode.md")]
    pub fn alpha_mode_py(&self, mode: AlphaMode) -> Self {
        self.alpha_mode(mode)
    }

    #[pyo3(name = "double_sided")]
    #[lsp_doc("docs/api/scene/unlit_material/double_sided.md")]
    pub fn double_sided_py(&self, value: bool) -> Self {
        self.double_sided(value)
    }

    #[pyo3(name = "base_color_map")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color_map.md")]
    pub fn base_color_map_py(&self) -> MaterialMap {
        self.base_color_map()
    }
}
