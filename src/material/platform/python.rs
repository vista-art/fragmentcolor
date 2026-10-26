#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::Shader;
use crate::material::{AlphaMode, Material, MaterialError, TextureSlot};
use crate::platforms::python::{read_floats, read_unsigned};
use crate::texture::PyTextureSource;

#[pymethods]
impl Material {
    #[staticmethod]
    #[pyo3(name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_py() -> Self {
        Material::pbr()
    }

    #[staticmethod]
    #[pyo3(name = "custom")]
    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom_py(shader: &Shader) -> Self {
        Material::custom(shader.clone())
    }

    #[pyo3(name = "shader")]
    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader_py(&self) -> Shader {
        // Returning a Clone of the inner Shader (Arc-share) is the natural
        // Python shape — the caller can keep operating on it directly. The
        // share semantics match Rust's `&Shader`: mutations on the returned
        // handle propagate back into the Material.
        self.shader.clone()
    }

    #[pyo3(name = "base_color")]
    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color_py(&self, color: [f32; 4]) -> Self {
        self.base_color(color)
    }

    #[pyo3(name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_py(&self, value: f32) -> Self {
        self.metallic(value)
    }

    #[pyo3(name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_py(&self, value: f32) -> Self {
        self.roughness(value)
    }

    #[pyo3(name = "normal_scale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_py(&self, value: f32) -> Self {
        self.normal_scale(value)
    }

    #[pyo3(name = "occlusion_strength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_py(&self, value: f32) -> Self {
        self.occlusion_strength(value)
    }

    #[pyo3(name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_py(&self, factor: [f32; 3]) -> Self {
        self.emissive(factor)
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: f32) -> Self {
        self.alpha_cutoff(value)
    }

    #[pyo3(name = "uv_transform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_py(
        &self,
        #[pyo3(from_py_with = read_floats::<2>)] offset: [f32; 2],
        #[pyo3(from_py_with = read_floats::<2>)] scale: [f32; 2],
        rotation: f32,
    ) -> Self {
        self.uv_transform(offset, scale, rotation)
    }

    #[pyo3(name = "texture_uv")]
    #[lsp_doc("docs/api/scene/material/texture_uv.md")]
    pub fn texture_uv_py(
        &self,
        slot: TextureSlot,
        #[pyo3(from_py_with = read_unsigned)] set: u32,
    ) -> PyResult<Self> {
        Ok(self.texture_uv(slot, set)?)
    }

    #[pyo3(name = "texture_transform")]
    #[lsp_doc("docs/api/scene/material/texture_transform.md")]
    pub fn texture_transform_py(
        &self,
        slot: TextureSlot,
        #[pyo3(from_py_with = read_floats::<2>)] offset: [f32; 2],
        #[pyo3(from_py_with = read_floats::<2>)] scale: [f32; 2],
        rotation: f32,
    ) -> Self {
        self.texture_transform(slot, offset, scale, rotation)
    }

    #[pyo3(name = "alpha_mode")]
    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode_py(&self, mode: AlphaMode) -> Self {
        self.alpha_mode(mode)
    }

    #[pyo3(name = "double_sided")]
    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided_py(&self, value: bool) -> Self {
        self.double_sided(value)
    }

    #[pyo3(name = "base_color_texture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_py(&self, texture: PyTextureSource<'_>) -> PyResult<Self> {
        Ok(self.base_color_texture(crate::TextureInput::try_from(texture)?))
    }

    #[pyo3(name = "metallic_roughness_texture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_py(&self, texture: PyTextureSource<'_>) -> PyResult<Self> {
        Ok(self.metallic_roughness_texture(crate::TextureInput::try_from(texture)?))
    }

    #[pyo3(name = "normal_texture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_py(&self, texture: PyTextureSource<'_>) -> PyResult<Self> {
        Ok(self.normal_texture(crate::TextureInput::try_from(texture)?))
    }

    #[pyo3(name = "occlusion_texture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_py(&self, texture: PyTextureSource<'_>) -> PyResult<Self> {
        Ok(self.occlusion_texture(crate::TextureInput::try_from(texture)?))
    }

    #[pyo3(name = "emissive_texture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_py(&self, texture: PyTextureSource<'_>) -> PyResult<Self> {
        Ok(self.emissive_texture(crate::TextureInput::try_from(texture)?))
    }
}

/// A value the material can't use raises `MaterialError`,
/// a `FragmentColorError` that is also a `ValueError`.
impl From<MaterialError> for PyErr {
    fn from(error: MaterialError) -> Self {
        crate::error::PythonError::Material.new_err(error.to_string())
    }
}
