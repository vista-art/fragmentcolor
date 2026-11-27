#![cfg(wasm)]

use crate::errors::WebError;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{read_finite, read_flag, read_vector};
use crate::core::read_alpha_mode;
use crate::{MaterialMap, Shader, UnlitMaterial};

#[wasm_bindgen]
impl UnlitMaterial {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/unlit_material/new.md")]
    pub fn new_js() -> UnlitMaterial {
        UnlitMaterial::new()
    }

    #[wasm_bindgen(js_name = "shader")]
    #[lsp_doc("docs/api/scene/unlit_material/shader.md")]
    pub fn shader_js(&self) -> Shader {
        self.shader.clone()
    }

    #[wasm_bindgen(js_name = "baseColor")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color.md")]
    pub fn base_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<UnlitMaterial, WebError> {
        let color = read_vector::<4>(color, "UnlitMaterial.baseColor")?;
        Ok(self.base_color(color)?)
    }

    #[wasm_bindgen(js_name = "alphaCutoff")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_cutoff.md")]
    pub fn alpha_cutoff_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] value: &JsValue,
    ) -> Result<UnlitMaterial, WebError> {
        Ok(self.alpha_cutoff(read_finite(value, "UnlitMaterial.alphaCutoff: value")?)?)
    }

    #[wasm_bindgen(js_name = "alphaMode")]
    #[lsp_doc("docs/api/scene/unlit_material/alpha_mode.md")]
    pub fn alpha_mode_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "AlphaMode")] mode: &JsValue,
    ) -> Result<UnlitMaterial, WebError> {
        Ok(self.alpha_mode(read_alpha_mode(mode)?))
    }

    #[wasm_bindgen(js_name = "doubleSided")]
    #[lsp_doc("docs/api/scene/unlit_material/double_sided.md")]
    pub fn double_sided_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "boolean")] value: &JsValue,
    ) -> Result<UnlitMaterial, WebError> {
        Ok(self.double_sided(read_flag(value, "UnlitMaterial.doubleSided: value")?))
    }

    #[wasm_bindgen(js_name = "baseColorMap")]
    #[lsp_doc("docs/api/scene/unlit_material/base_color_map.md")]
    pub fn base_color_map_js(&self) -> MaterialMap {
        self.base_color_map()
    }
}

crate::impl_js_bridge!(UnlitMaterial, crate::PassError);
