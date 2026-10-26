#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::Shader;
use crate::material::{AlphaMode, Material, TextureSlot};
use crate::platforms::web::read_vector;
use crate::texture::read_texture_input;

#[wasm_bindgen]
impl Material {
    #[wasm_bindgen(js_name = "pbr")]
    #[lsp_doc("docs/api/scene/material/pbr.md")]
    pub fn pbr_js() -> Material {
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
    pub fn base_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<Material, JsError> {
        let arr = read_vector::<4>(color, "Material.baseColor")?;
        Ok(self.base_color(arr))
    }

    #[wasm_bindgen(js_name = "metallic")]
    #[lsp_doc("docs/api/scene/material/metallic.md")]
    pub fn metallic_js(&self, value: f32) -> Self {
        self.metallic(value)
    }

    #[wasm_bindgen(js_name = "roughness")]
    #[lsp_doc("docs/api/scene/material/roughness.md")]
    pub fn roughness_js(&self, value: f32) -> Self {
        self.roughness(value)
    }

    #[wasm_bindgen(js_name = "normalScale")]
    #[lsp_doc("docs/api/scene/material/normal_scale.md")]
    pub fn normal_scale_js(&self, value: f32) -> Self {
        self.normal_scale(value)
    }

    #[wasm_bindgen(js_name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/material/occlusion_strength.md")]
    pub fn occlusion_strength_js(&self, value: f32) -> Self {
        self.occlusion_strength(value)
    }

    #[wasm_bindgen(js_name = "emissive")]
    #[lsp_doc("docs/api/scene/material/emissive.md")]
    pub fn emissive_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] factor: &JsValue,
    ) -> Result<Material, JsError> {
        let arr = read_vector::<3>(factor, "Material.emissive")?;
        Ok(self.emissive(arr))
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/material/alpha_cutoff.md")]
    pub fn alpha_cutoff_js(&self, value: f32) -> Self {
        self.alpha_cutoff(value)
    }

    #[wasm_bindgen(js_name = "uvTransform")]
    #[lsp_doc("docs/api/scene/material/uv_transform.md")]
    pub fn uv_transform_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] offset: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] scale: &JsValue,
        rotation: f32,
    ) -> Result<Material, JsError> {
        let o = read_vector::<2>(offset, "Material.uvTransform: offset")?;
        let s = read_vector::<2>(scale, "Material.uvTransform: scale")?;
        Ok(self.uv_transform(o, s, rotation))
    }

    #[wasm_bindgen(js_name = "textureUv")]
    #[lsp_doc("docs/api/scene/material/texture_uv.md")]
    pub fn texture_uv_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "TextureSlot")] slot: &JsValue,
        set: f64,
    ) -> Result<Material, JsError> {
        let set = crate::platforms::web::read_unsigned(set, "Material.textureUv: set")?;
        Ok(self.texture_uv(read_texture_slot(slot)?, set)?)
    }

    #[wasm_bindgen(js_name = "textureTransform")]
    #[lsp_doc("docs/api/scene/material/texture_transform.md")]
    pub fn texture_transform_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "TextureSlot")] slot: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] offset: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] scale: &JsValue,
        rotation: f32,
    ) -> Result<Material, JsError> {
        let o = read_vector::<2>(offset, "Material.textureTransform: offset")?;
        let s = read_vector::<2>(scale, "Material.textureTransform: scale")?;
        Ok(self.texture_transform(read_texture_slot(slot)?, o, s, rotation))
    }

    #[wasm_bindgen(js_name = "alphaMode")]
    #[lsp_doc("docs/api/scene/material/alpha_mode.md")]
    pub fn alpha_mode_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "AlphaMode")] mode: &JsValue,
    ) -> Result<Material, JsError> {
        Ok(self.alpha_mode(read_alpha_mode(mode)?))
    }

    #[wasm_bindgen(js_name = "doubleSided")]
    #[lsp_doc("docs/api/scene/material/double_sided.md")]
    pub fn double_sided_js(&self, value: bool) -> Self {
        self.double_sided(value)
    }

    #[wasm_bindgen(js_name = "baseColorTexture")]
    #[lsp_doc("docs/api/scene/material/base_color_texture.md")]
    pub fn base_color_texture_js(
        &self,
        source: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Self, JsError> {
        Ok(self.base_color_texture(read_texture_input(source, options.as_ref())?))
    }

    #[wasm_bindgen(js_name = "metallicRoughnessTexture")]
    #[lsp_doc("docs/api/scene/material/metallic_roughness_texture.md")]
    pub fn metallic_roughness_texture_js(
        &self,
        source: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Self, JsError> {
        Ok(self.metallic_roughness_texture(read_texture_input(source, options.as_ref())?))
    }

    #[wasm_bindgen(js_name = "normalTexture")]
    #[lsp_doc("docs/api/scene/material/normal_texture.md")]
    pub fn normal_texture_js(
        &self,
        source: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Self, JsError> {
        Ok(self.normal_texture(read_texture_input(source, options.as_ref())?))
    }

    #[wasm_bindgen(js_name = "occlusionTexture")]
    #[lsp_doc("docs/api/scene/material/occlusion_texture.md")]
    pub fn occlusion_texture_js(
        &self,
        source: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Self, JsError> {
        Ok(self.occlusion_texture(read_texture_input(source, options.as_ref())?))
    }

    #[wasm_bindgen(js_name = "emissiveTexture")]
    #[lsp_doc("docs/api/scene/material/emissive_texture.md")]
    pub fn emissive_texture_js(
        &self,
        source: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Self, JsError> {
        Ok(self.emissive_texture(read_texture_input(source, options.as_ref())?))
    }
}

/// Maps an `AlphaMode` from JS, a number in variant order, to the mode.
/// Anything else is an error, which wasm-bindgen would read as `Opaque`.
fn read_alpha_mode(value: &JsValue) -> Result<AlphaMode, JsError> {
    let code = value
        .as_f64()
        .ok_or_else(|| JsError::new(&format!("mode must be an AlphaMode, got {value:?}")))?;
    match crate::platforms::web::read_unsigned(code, "mode")? {
        0 => Ok(AlphaMode::Opaque),
        1 => Ok(AlphaMode::Mask),
        2 => Ok(AlphaMode::Blend),
        _ => Err(JsError::new(&format!("{code} is not an AlphaMode"))),
    }
}

/// Maps a `TextureSlot` from JS, a number in variant order, to the slot.
/// Anything else is an error, which wasm-bindgen would read as `BaseColor`.
fn read_texture_slot(value: &JsValue) -> Result<TextureSlot, JsError> {
    let code = value
        .as_f64()
        .ok_or_else(|| JsError::new(&format!("slot must be a TextureSlot, got {value:?}")))?;
    let index = crate::platforms::web::read_unsigned(code, "slot")?;
    TextureSlot::ALL
        .get(index as usize)
        .copied()
        .ok_or_else(|| JsError::new(&format!("{code} is not a TextureSlot")))
}
