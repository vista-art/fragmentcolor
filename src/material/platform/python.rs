#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::Shader;
use crate::bindings::python::{PyChannels, PyFloat, PyUnsigned};
use crate::material::{AlphaMode, Material, MaterialError, TextureSlot};
use crate::renderer::platform::python::{TextureKeywords, py_to_texture_spec};
use crate::texture::{PySamplerOptions, PyTextureSource, PyTextureUsage};
use crate::{PySize, TextureFormat};

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
    pub fn base_color_py(&self, color: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.base_color(color.read_array("Material.base_color")?))
    }

    #[pyo3(name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.metallic(value.read("Material.metallic: value")?))
    }

    #[pyo3(name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.roughness(value.read("Material.roughness: value")?))
    }

    #[pyo3(name = "normal_scale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.normal_scale(value.read("Material.normal_scale: value")?))
    }

    #[pyo3(name = "occlusion_strength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.occlusion_strength(value.read("Material.occlusion_strength: value")?))
    }

    #[pyo3(name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_py(&self, factor: PyChannels<'_>) -> PyResult<Self> {
        Ok(self.emissive(factor.read_array("Material.emissive")?))
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: PyFloat<'_>) -> PyResult<Self> {
        Ok(self.alpha_cutoff(value.read("Material.alpha_cutoff: value")?))
    }

    #[pyo3(name = "uv_transform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_py(
        &self,
        offset: PyChannels<'_>,
        scale: PyChannels<'_>,
        rotation: PyFloat<'_>,
    ) -> PyResult<Self> {
        let offset = offset.read_array("Material.uv_transform: offset")?;
        let scale = scale.read_array("Material.uv_transform: scale")?;
        let rotation = rotation.read("Material.uv_transform: rotation")?;
        Ok(self.uv_transform(offset, scale, rotation))
    }

    #[pyo3(name = "texture_uv")]
    #[lsp_doc("docs/api/scene/material/texture_uv.md")]
    pub fn texture_uv_py(&self, slot: TextureSlot, set: PyUnsigned<'_>) -> PyResult<Self> {
        Ok(self.texture_uv(slot, set.read("Material.texture_uv: set")?)?)
    }

    #[pyo3(name = "texture_transform")]
    #[lsp_doc("docs/api/scene/material/texture_transform.md")]
    pub fn texture_transform_py(
        &self,
        slot: TextureSlot,
        offset: PyChannels<'_>,
        scale: PyChannels<'_>,
        rotation: PyFloat<'_>,
    ) -> PyResult<Self> {
        let offset = offset.read_array("Material.texture_transform: offset")?;
        let scale = scale.read_array("Material.texture_transform: scale")?;
        let rotation = rotation.read("Material.texture_transform: rotation")?;
        Ok(self.texture_transform(slot, offset, scale, rotation))
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

    #[pyo3(
        name = "base_color_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let source = py_to_texture_spec(input, keywords, "Material.base_color_texture")?;
        Ok(self.base_color_texture(source))
    }

    #[pyo3(
        name = "metallic_roughness_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let source = py_to_texture_spec(input, keywords, "Material.metallic_roughness_texture")?;
        Ok(self.metallic_roughness_texture(source))
    }

    #[pyo3(
        name = "normal_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let source = py_to_texture_spec(input, keywords, "Material.normal_texture")?;
        Ok(self.normal_texture(source))
    }

    #[pyo3(
        name = "occlusion_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let source = py_to_texture_spec(input, keywords, "Material.occlusion_texture")?;
        Ok(self.occlusion_texture(source))
    }

    #[pyo3(
        name = "emissive_texture",
        signature = (input, size=None, format=None, mipmaps=None, sampler=None, usage=None)
    )]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_py<'py>(
        &self,
        input: PyTextureSource<'py>,
        size: Option<PySize<'py>>,
        format: Option<TextureFormat>,
        mipmaps: Option<bool>,
        sampler: Option<PySamplerOptions<'py>>,
        usage: Option<PyTextureUsage<'py>>,
    ) -> PyResult<Self> {
        let keywords = TextureKeywords::new(size, format, mipmaps, sampler, usage);
        let source = py_to_texture_spec(input, keywords, "Material.emissive_texture")?;
        Ok(self.emissive_texture(source))
    }
}

/// A value the material can't use raises `MaterialError`,
/// a `FragmentColorError` that is also a `ValueError`.
impl From<MaterialError> for PyErr {
    fn from(error: MaterialError) -> Self {
        crate::error::raise_error(&error)
    }
}
