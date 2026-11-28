#![cfg(wasm)]

use crate::errors::WebError;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{read_count, read_finite, read_flag, read_vector};
use crate::core::read_alpha_mode;
use crate::{MaterialMap, Shader, ToonMaterial};

#[wasm_bindgen]
impl ToonMaterial {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/toon_material/new.md")]
    pub fn new_js() -> ToonMaterial {
        ToonMaterial::new()
    }

    #[wasm_bindgen(js_name = "shader")]
    #[lsp_doc("docs/api/scene/toon_material/shader.md")]
    pub fn shader_js(&self) -> Shader {
        self.shader.clone()
    }

    #[wasm_bindgen(js_name = "baseColor")]
    #[lsp_doc("docs/api/scene/toon_material/base_color.md")]
    pub fn base_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        let color = read_vector::<4>(color, "ToonMaterial.baseColor")?;
        Ok(self.base_color(color)?)
    }

    #[wasm_bindgen(js_name = "bands")]
    #[lsp_doc("docs/api/scene/toon_material/bands.md")]
    pub fn bands_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] count: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        Ok(self.bands(read_count(count, "ToonMaterial.bands: count")?)?)
    }

    #[wasm_bindgen(js_name = "highlight")]
    #[lsp_doc("docs/api/scene/toon_material/highlight.md")]
    pub fn highlight_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        let color = read_vector::<3>(color, "ToonMaterial.highlight")?;
        Ok(self.highlight(color)?)
    }

    #[wasm_bindgen(js_name = "highlightSize")]
    #[lsp_doc("docs/api/scene/toon_material/highlight_size.md")]
    pub fn highlight_size_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        let value = read_finite(value, "ToonMaterial.highlightSize: value")?;
        Ok(self.highlight_size(value)?)
    }

    #[wasm_bindgen(js_name = "rim")]
    #[lsp_doc("docs/api/scene/toon_material/rim.md")]
    pub fn rim_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        let color = read_vector::<3>(color, "ToonMaterial.rim")?;
        Ok(self.rim(color)?)
    }

    #[wasm_bindgen(js_name = "rimWidth")]
    #[lsp_doc("docs/api/scene/toon_material/rim_width.md")]
    pub fn rim_width_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        Ok(self.rim_width(read_finite(value, "ToonMaterial.rimWidth: value")?)?)
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        Ok(self.alpha_cutoff(read_finite(value, "ToonMaterial.alphaCutoff: value")?)?)
    }

    #[wasm_bindgen(js_name = "alphaMode")]
    #[lsp_doc("docs/api/scene/toon_material/alpha_mode.md")]
    pub fn alpha_mode_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "AlphaMode")] mode: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        Ok(self.alpha_mode(read_alpha_mode(mode, "ToonMaterial.alphaMode: mode")?))
    }

    #[wasm_bindgen(js_name = "doubleSided")]
    #[lsp_doc("docs/api/scene/toon_material/double_sided.md")]
    pub fn double_sided_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: &JsValue,
    ) -> Result<ToonMaterial, WebError> {
        Ok(self.double_sided(read_flag(value, "ToonMaterial.doubleSided: value")?))
    }

    #[wasm_bindgen(js_name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/toon_material/base_color_map.md")]
    pub fn base_color_map_js(&self) -> MaterialMap {
        self.base_color_map()
    }

    #[wasm_bindgen(js_name = "rampMap")]
    #[lsp_doc("docs/api/scene/toon_material/ramp_map.md")]
    pub fn ramp_map_js(&self) -> MaterialMap {
        self.ramp_map()
    }
}

crate::impl_js_bridge!(ToonMaterial, crate::PassError);
