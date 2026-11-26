#![cfg(wasm)]

use crate::errors::WebError;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{read_finite, read_flag, read_vector};
use crate::scene::material::platform::web::read_alpha_mode;
use crate::{MaterialMap, PbrMaterial, Shader};

#[wasm_bindgen]
impl PbrMaterial {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/pbr_material/new.md")]
    pub fn new_js() -> PbrMaterial {
        PbrMaterial::new()
    }

    #[wasm_bindgen(js_name = "shader")]
    #[lsp_doc("docs/api/scene/pbr_material/shader.md")]
    pub fn shader_js(&self) -> Shader {
        self.shader.clone()
    }

    #[wasm_bindgen(js_name = "baseColor")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color.md")]
    pub fn base_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        let color = read_vector::<4>(color, "PbrMaterial.baseColor")?;
        Ok(self.base_color(color)?)
    }

    #[wasm_bindgen(js_name = "metallic")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic.md")]
    pub fn metallic_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.metallic(read_finite(value, "PbrMaterial.metallic: value")?)?)
    }

    #[wasm_bindgen(js_name = "roughness")]
    #[lsp_doc("docs/api/scene/pbr_material/roughness.md")]
    pub fn roughness_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.roughness(read_finite(value, "PbrMaterial.roughness: value")?)?)
    }

    #[wasm_bindgen(js_name = "normalScale")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_scale.md")]
    pub fn normal_scale_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.normal_scale(read_finite(value, "PbrMaterial.normalScale: value")?)?)
    }

    #[wasm_bindgen(js_name = "occlusionStrength")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_strength.md")]
    pub fn occlusion_strength_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        let value = read_finite(value, "PbrMaterial.occlusionStrength: value")?;
        Ok(self.occlusion_strength(value)?)
    }

    #[wasm_bindgen(js_name = "emissive")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive.md")]
    pub fn emissive_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] factor: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        let factor = read_vector::<3>(factor, "PbrMaterial.emissive")?;
        Ok(self.emissive(factor)?)
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.alpha_cutoff(read_finite(value, "PbrMaterial.alphaCutoff: value")?)?)
    }

    #[wasm_bindgen(js_name = "uvTransform")]
    #[lsp_doc("docs/api/scene/pbr_material/uv_transform.md")]
    pub fn uv_transform_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] offset: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] scale: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] rotation: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        let offset = read_vector::<2>(offset, "PbrMaterial.uvTransform: offset")?;
        let scale = read_vector::<2>(scale, "PbrMaterial.uvTransform: scale")?;
        let rotation = read_finite(rotation, "PbrMaterial.uvTransform: rotation")?;
        Ok(self.uv_transform(offset, scale, rotation)?)
    }

    #[wasm_bindgen(js_name = "alphaMode")]
    #[lsp_doc("docs/api/scene/pbr_material/alpha_mode.md")]
    pub fn alpha_mode_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "AlphaMode")] mode: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.alpha_mode(read_alpha_mode(mode)?))
    }

    #[wasm_bindgen(js_name = "doubleSided")]
    #[lsp_doc("docs/api/scene/pbr_material/double_sided.md")]
    pub fn double_sided_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: &JsValue,
    ) -> Result<PbrMaterial, WebError> {
        Ok(self.double_sided(read_flag(value, "PbrMaterial.doubleSided: value")?))
    }

    #[wasm_bindgen(js_name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/pbr_material/base_color_map.md")]
    pub fn base_color_map_js(&self) -> MaterialMap {
        self.base_color_map()
    }

    #[wasm_bindgen(js_name = "metallicRoughnessMap")]
    #[lsp_doc("docs/api/scene/pbr_material/metallic_roughness_map.md")]
    pub fn metallic_roughness_map_js(&self) -> MaterialMap {
        self.metallic_roughness_map()
    }

    #[wasm_bindgen(js_name = "normalMap")]
    #[lsp_doc("docs/api/scene/pbr_material/normal_map.md")]
    pub fn normal_map_js(&self) -> MaterialMap {
        self.normal_map()
    }

    #[wasm_bindgen(js_name = "occlusionMap")]
    #[lsp_doc("docs/api/scene/pbr_material/occlusion_map.md")]
    pub fn occlusion_map_js(&self) -> MaterialMap {
        self.occlusion_map()
    }

    #[wasm_bindgen(js_name = "emissiveMap")]
    #[lsp_doc("docs/api/scene/pbr_material/emissive_map.md")]
    pub fn emissive_map_js(&self) -> MaterialMap {
        self.emissive_map()
    }
}

crate::impl_js_bridge!(PbrMaterial, crate::PassError);
