#![cfg(python)]

use lsp_doc::lsp_doc;
use pyo3::prelude::*;

use crate::material::{AlphaMode, Material, set_or_warn, set_texture_or_warn};
use crate::Shader;

#[pymethods]
impl Material {
    #[staticmethod]
    #[pyo3(name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_py() -> Result<Self, PyErr> {
        Material::pbr().map_err(|e| e.into())
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
    pub fn base_color_py(&self, color: [f32; 4]) {
        set_or_warn(&self.shader, "material.base_color", color);
    }

    #[pyo3(name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_py(&self, value: f32) {
        set_or_warn(&self.shader, "material.metallic", value);
    }

    #[pyo3(name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_py(&self, value: f32) {
        set_or_warn(&self.shader, "material.roughness", value);
    }

    #[pyo3(name = "normal_scale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_py(&self, value: f32) {
        set_or_warn(&self.shader, "material.normal_scale", value);
    }

    #[pyo3(name = "occlusion_strength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_py(&self, value: f32) {
        set_or_warn(&self.shader, "material.occlusion_strength", value);
    }

    #[pyo3(name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_py(&self, factor: [f32; 3]) {
        set_or_warn(&self.shader, "material.emissive", factor);
    }

    #[pyo3(name = "alpha_cutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_py(&self, value: f32) {
        set_or_warn(&self.shader, "material.alpha_cutoff", value);
    }

    #[pyo3(name = "uv_transform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_py(&self, offset: [f32; 2], scale: [f32; 2], rotation: f32) {
        set_or_warn(&self.shader, "material.uv_offset", offset);
        set_or_warn(&self.shader, "material.uv_scale", scale);
        set_or_warn(&self.shader, "material.uv_rotation", rotation);
    }

    #[pyo3(name = "alpha_mode")]
    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode_py(&self, mode: AlphaMode) {
        *self.alpha_mode.write() = mode;
        self.shader.object.set_alpha_mode(mode);
        set_or_warn(&self.shader, "material.alpha_mode_flag", mode.flag());
    }

    #[pyo3(name = "double_sided")]
    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided_py(&self, value: bool) {
        *self.double_sided.write() = value;
        self.shader.object.set_double_sided(value);
    }

    #[pyo3(name = "base_color_texture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_py(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "base_color_map", texture);
    }

    #[pyo3(name = "metallic_roughness_texture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_py(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "metallic_roughness_map", texture);
    }

    #[pyo3(name = "normal_texture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_py(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "normal_map", texture);
    }

    #[pyo3(name = "occlusion_texture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_py(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "occlusion_map", texture);
    }

    #[pyo3(name = "emissive_texture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_py(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "emissive_map", texture);
    }
}
