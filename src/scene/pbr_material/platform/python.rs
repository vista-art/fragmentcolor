#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::bindings::python::{PyChannels, PyFloat};
use crate::{AlphaMode, MaterialMap, PbrMaterial, Shader};

#[pymethods]
impl PbrMaterial {
    #[new]
    #[lsp_doc("docs/api/scene/pbr_material/new.md")]
    pub fn new_py() -> Self {
        PbrMaterial::new()
    }

    #[pyo3(name = "shader")]
    #[lsp_doc("docs/api/scene/pbr_material/shader.md")]
    pub fn shader_py(&self) -> Shader {
        // A clone of the inner Shader is an Arc-share, so the caller works on the
        // material's own shader, as Rust's `&Shader` does.
        self.shader.clone()
    }

    #[pyo3(name = "base_color")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color.md")]
    pub fn base_color_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.base_color(color.read_array("PbrMaterial.base_color")?)?)
    }

    #[pyo3(name = "metallic")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic.md")]
    pub fn metallic_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.metallic(value.read_finite("PbrMaterial.metallic: value")?)?)
    }

    #[pyo3(name = "roughness")]
    #[lsp_doc("docs/api/scene/pbr_material/roughness.md")]
    pub fn roughness_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.roughness(value.read_finite("PbrMaterial.roughness: value")?)?)
    }

    #[pyo3(name = "normal_scale")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_scale.md")]
    pub fn normal_scale_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.normal_scale(value.read_finite("PbrMaterial.normal_scale: value")?)?)
    }

    #[pyo3(name = "occlusion_strength")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_strength.md")]
    pub fn occlusion_strength_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        let value = value.read_finite("PbrMaterial.occlusion_strength: value")?;
        Ok(self.occlusion_strength(value)?)
    }

    #[pyo3(name = "emissive")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive.md")]
    pub fn emissive_py(&self, factor: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.emissive(factor.read_array("PbrMaterial.emissive")?)?)
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.alpha_cutoff(value.read_finite("PbrMaterial.alpha_cutoff: value")?)?)
    }

    #[pyo3(name = "uv_transform")]
    #[lsp_doc("docs/api/scene/pbr_material/uv_transform.md")]
    pub fn uv_transform_py(
        &self,
        offset: PyChannels<'_>,
        scale: PyChannels<'_>,
        rotation: PyFloat<'_>,
    ) -> PyResult<Self> {
        let offset = offset.read_array("PbrMaterial.uv_transform: offset")?;
        let scale = scale.read_array("PbrMaterial.uv_transform: scale")?;
        let rotation = rotation.read_finite("PbrMaterial.uv_transform: rotation")?;
        Ok(self.uv_transform(offset, scale, rotation)?)
    }

    #[pyo3(name = "alpha_mode")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_mode.md")]
    pub fn alpha_mode_py(&self, mode: AlphaMode) -> Self {
        self.alpha_mode(mode)
    }

    #[pyo3(name = "double_sided")]
    #[lsp_doc("docs/api/scene/pbr_material/double_sided.md")]
    pub fn double_sided_py(&self, value: bool) -> Self {
        self.double_sided(value)
    }

    #[pyo3(name = "base_color_map")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color_map.md")]
    pub fn base_color_map_py(&self) -> MaterialMap {
        self.base_color_map()
    }

    #[pyo3(name = "metallic_roughness_map")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic_roughness_map.md")]
    pub fn metallic_roughness_map_py(&self) -> MaterialMap {
        self.metallic_roughness_map()
    }

    #[pyo3(name = "normal_map")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_map.md")]
    pub fn normal_map_py(&self) -> MaterialMap {
        self.normal_map()
    }

    #[pyo3(name = "occlusion_map")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_map.md")]
    pub fn occlusion_map_py(&self) -> MaterialMap {
        self.occlusion_map()
    }

    #[pyo3(name = "emissive_map")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive_map.md")]
    pub fn emissive_map_py(&self) -> MaterialMap {
        self.emissive_map()
    }
}
