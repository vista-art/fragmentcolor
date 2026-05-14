#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::material::{AlphaMode, Material, set_or_warn, set_texture_or_warn};
use crate::renderer::platform::mobile::FragmentColorError;
use crate::texture::Texture;
use crate::Shader;

#[uniffi::export]
impl Material {
    #[uniffi::constructor(name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_mobile() -> Result<Arc<Self>, FragmentColorError> {
        Material::pbr()
            .map(Arc::new)
            .map_err(|e| FragmentColorError::Render(e.to_string()))
    }

    #[uniffi::constructor(name = "custom")]
    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom_mobile(shader: Arc<Shader>) -> Arc<Self> {
        Arc::new(Material::custom((*shader).clone()))
    }

    #[uniffi::method(name = "shader")]
    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader_mobile(self: Arc<Self>) -> Arc<Shader> {
        Arc::new(self.shader.clone())
    }

    #[uniffi::method(name = "baseColor")]
    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color_mobile(self: Arc<Self>, color: Vec<f32>) -> Result<(), FragmentColorError> {
        let arr = take_vec4(&color, "base_color")?;
        set_or_warn(&self.shader, "material.base_color", arr);
        Ok(())
    }

    #[uniffi::method(name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_mobile(self: Arc<Self>, value: f32) {
        set_or_warn(&self.shader, "material.metallic", value);
    }

    #[uniffi::method(name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_mobile(self: Arc<Self>, value: f32) {
        set_or_warn(&self.shader, "material.roughness", value);
    }

    #[uniffi::method(name = "normalScale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_mobile(self: Arc<Self>, value: f32) {
        set_or_warn(&self.shader, "material.normal_scale", value);
    }

    #[uniffi::method(name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_mobile(self: Arc<Self>, value: f32) {
        set_or_warn(&self.shader, "material.occlusion_strength", value);
    }

    #[uniffi::method(name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_mobile(
        self: Arc<Self>,
        factor: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&factor, "emissive")?;
        set_or_warn(&self.shader, "material.emissive", arr);
        Ok(())
    }

    #[uniffi::method(name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_mobile(self: Arc<Self>, value: f32) {
        set_or_warn(&self.shader, "material.alpha_cutoff", value);
    }

    #[uniffi::method(name = "uvTransform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
        scale: Vec<f32>,
        rotation: f32,
    ) -> Result<(), FragmentColorError> {
        let o = take_vec2(&offset, "uvTransform offset")?;
        let s = take_vec2(&scale, "uvTransform scale")?;
        set_or_warn(&self.shader, "material.uv_offset", o);
        set_or_warn(&self.shader, "material.uv_scale", s);
        set_or_warn(&self.shader, "material.uv_rotation", rotation);
        Ok(())
    }

    #[uniffi::method(name = "alphaMode")]
    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode_mobile(self: Arc<Self>, mode: AlphaMode) {
        *self.alpha_mode.write() = mode;
        self.shader.object.set_alpha_mode(mode);
        set_or_warn(&self.shader, "material.alpha_mode_flag", mode.flag());
    }

    #[uniffi::method(name = "doubleSided")]
    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided_mobile(self: Arc<Self>, value: bool) {
        *self.double_sided.write() = value;
        self.shader.object.set_double_sided(value);
    }

    #[uniffi::method(name = "baseColorTexture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) {
        set_texture_or_warn(&self.shader, "base_color_map", &texture);
    }

    #[uniffi::method(name = "metallicRoughnessTexture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) {
        set_texture_or_warn(&self.shader, "metallic_roughness_map", &texture);
    }

    #[uniffi::method(name = "normalTexture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) {
        set_texture_or_warn(&self.shader, "normal_map", &texture);
    }

    #[uniffi::method(name = "occlusionTexture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) {
        set_texture_or_warn(&self.shader, "occlusion_map", &texture);
    }

    #[uniffi::method(name = "emissiveTexture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_mobile(self: Arc<Self>, texture: Arc<Texture>) {
        set_texture_or_warn(&self.shader, "emissive_map", &texture);
    }
}

fn take_vec2(v: &[f32], field: &str) -> Result<[f32; 2], FragmentColorError> {
    if v.len() != 2 {
        return Err(FragmentColorError::Render(format!(
            "Material.{field}: expected an array of length 2, got {}",
            v.len()
        )));
    }
    Ok([v[0], v[1]])
}

fn take_vec3(v: &[f32], field: &str) -> Result<[f32; 3], FragmentColorError> {
    if v.len() != 3 {
        return Err(FragmentColorError::Render(format!(
            "Material.{field}: expected an array of length 3, got {}",
            v.len()
        )));
    }
    Ok([v[0], v[1], v[2]])
}

fn take_vec4(v: &[f32], field: &str) -> Result<[f32; 4], FragmentColorError> {
    if v.len() != 4 {
        return Err(FragmentColorError::Render(format!(
            "Material.{field}: expected an array of length 4, got {}",
            v.len()
        )));
    }
    Ok([v[0], v[1], v[2], v[3]])
}
