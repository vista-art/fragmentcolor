#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::bindings::python::{PyAlphaMode, PyChannels, PyFloat, PyUnsigned};
use crate::{MaterialMap, Shader, ToonMaterial};

#[pymethods]
impl ToonMaterial {
    #[new]
    #[lsp_doc("docs/api/scene/toon_material/new.md")]
    pub fn new_py() -> Self {
        ToonMaterial::new()
    }

    #[pyo3(name = "shader")]
    #[lsp_doc("docs/api/scene/toon_material/shader.md")]
    pub fn shader_py(&self) -> Shader {
        // A clone of the inner Shader is an Arc-share, so the caller works on the
        // material's own shader, as Rust's `&Shader` does.
        self.shader.clone()
    }

    #[pyo3(name = "base_color")]
    #[lsp_doc("docs/api/scene/toon_material/base_color.md")]
    pub fn base_color_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.base_color(color.read_array("ToonMaterial.base_color")?)?)
    }

    #[pyo3(name = "bands")]
    #[lsp_doc("docs/api/scene/toon_material/bands.md")]
    pub fn bands_py(&self, count: PyUnsigned<'_>) -> PyResult<Self> {
        Ok(self.bands(count.read("ToonMaterial.bands: count")?)?)
    }

    #[pyo3(name = "highlight")]
    #[lsp_doc("docs/api/scene/toon_material/highlight.md")]
    pub fn highlight_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.highlight(color.read_array("ToonMaterial.highlight")?)?)
    }

    #[pyo3(name = "highlight_size")]
    #[lsp_doc("docs/api/scene/toon_material/highlight_size.md")]
    pub fn highlight_size_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        let value = value.read_finite("ToonMaterial.highlight_size: value")?;
        Ok(self.highlight_size(value)?)
    }

    #[pyo3(name = "rim")]
    #[lsp_doc("docs/api/scene/toon_material/rim.md")]
    pub fn rim_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.rim(color.read_array("ToonMaterial.rim")?)?)
    }

    #[pyo3(name = "rim_width")]
    #[lsp_doc("docs/api/scene/toon_material/rim_width.md")]
    pub fn rim_width_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.rim_width(value.read_finite("ToonMaterial.rim_width: value")?)?)
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.alpha_cutoff(value.read_finite("ToonMaterial.alpha_cutoff: value")?)?)
    }

    #[pyo3(name = "alpha_mode")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_mode.md")]
    pub fn alpha_mode_py(&self, mode: PyAlphaMode<'_>) -> PyResult<Self> {
        Ok(self.alpha_mode(mode.read("ToonMaterial.alpha_mode: mode")?))
    }

    #[pyo3(name = "double_sided")]
    #[lsp_doc("docs/api/scene/toon_material/double_sided.md")]
    pub fn double_sided_py(&self, value: bool) -> Self {
        self.double_sided(value)
    }

    #[pyo3(name = "base_color_map")]
    #[lsp_doc("docs/api/scene/toon_material/base_color_map.md")]
    pub fn base_color_map_py(&self) -> MaterialMap {
        self.base_color_map()
    }

    #[pyo3(name = "ramp_map")]
    #[lsp_doc("docs/api/scene/toon_material/ramp_map.md")]
    pub fn ramp_map_py(&self) -> MaterialMap {
        self.ramp_map()
    }
}
