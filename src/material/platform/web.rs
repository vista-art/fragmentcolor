#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::material::{Material, set_or_warn, set_texture_or_warn};
use crate::Shader;

#[wasm_bindgen]
impl Material {
    #[wasm_bindgen(js_name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_js() -> Self {
        Material::pbr()
    }

    #[wasm_bindgen(js_name = "custom")]
    #[lsp_doc("docs/api/scene/material/custom.md")]
    pub fn custom_js(shader: &Shader) -> Self {
        Material::custom(shader.clone())
    }

    #[wasm_bindgen(js_name = "shader")]
    #[lsp_doc("docs/api/scene/material/shader.md")]
    pub fn shader_js(&self) -> Shader {
        self.shader.clone()
    }

    #[wasm_bindgen(js_name = "baseColor")]
    #[lsp_doc("docs/api/scene/material/base_color.md")]
    pub fn base_color_js(&self, color: Vec<f32>) -> Result<(), JsError> {
        let arr = vec4_from_vec(&color, "base_color")?;
        set_or_warn(&self.shader, "material.base_color", arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_js(&self, value: f32) {
        set_or_warn(&self.shader, "material.metallic", value);
    }

    #[wasm_bindgen(js_name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_js(&self, value: f32) {
        set_or_warn(&self.shader, "material.roughness", value);
    }

    #[wasm_bindgen(js_name = "normalScale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_js(&self, value: f32) {
        set_or_warn(&self.shader, "material.normal_scale", value);
    }

    #[wasm_bindgen(js_name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_js(&self, value: f32) {
        set_or_warn(&self.shader, "material.occlusion_strength", value);
    }

    #[wasm_bindgen(js_name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_js(&self, factor: Vec<f32>) -> Result<(), JsError> {
        let arr = vec3_from_vec(&factor, "emissive")?;
        set_or_warn(&self.shader, "material.emissive", arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_js(&self, value: f32) {
        set_or_warn(&self.shader, "material.alpha_cutoff", value);
    }

    #[wasm_bindgen(js_name = "baseColorTexture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_js(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "base_color_map", texture);
    }

    #[wasm_bindgen(js_name = "metallicRoughnessTexture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_js(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "metallic_roughness_map", texture);
    }

    #[wasm_bindgen(js_name = "normalTexture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_js(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "normal_map", texture);
    }

    #[wasm_bindgen(js_name = "occlusionTexture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_js(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "occlusion_map", texture);
    }

    #[wasm_bindgen(js_name = "emissiveTexture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_js(&self, texture: &crate::texture::Texture) {
        set_texture_or_warn(&self.shader, "emissive_map", texture);
    }
}

fn vec3_from_vec(v: &[f32], field: &str) -> Result<[f32; 3], JsError> {
    if v.len() != 3 {
        return Err(JsError::new(&format!(
            "Material.{field}: expected an array of length 3"
        )));
    }
    Ok([v[0], v[1], v[2]])
}

fn vec4_from_vec(v: &[f32], field: &str) -> Result<[f32; 4], JsError> {
    if v.len() != 4 {
        return Err(JsError::new(&format!(
            "Material.{field}: expected an array of length 4"
        )));
    }
    Ok([v[0], v[1], v[2], v[3]])
}
