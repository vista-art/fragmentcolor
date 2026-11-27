#![cfg(wasm)]

use crate::errors::WebError;
use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{read_count, read_finite, read_vector};
use crate::texture::{read_texture_input, refuse_options};
use crate::{MaterialMap, TextureData, TextureInput};

#[wasm_bindgen]
impl MaterialMap {
    #[wasm_bindgen(js_name = "texture")]
    #[lsp_doc("docs/api/scene/material_map/texture.md")]
    pub fn texture_js(&self, input: &JsValue, options: Option<JsValue>) -> Result<Self, WebError> {
        let source = read_input(input, options.as_ref(), "MaterialMap.texture")?;
        Ok(self.texture(source)?)
    }

    #[wasm_bindgen(js_name = "uv")]
    #[lsp_doc("docs/api/scene/material_map/uv.md")]
    pub fn uv_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] set: &JsValue,
    ) -> Result<Self, WebError> {
        Ok(self.uv(read_count(set, "MaterialMap.uv: set")?)?)
    }

    #[wasm_bindgen(js_name = "transform")]
    #[lsp_doc("docs/api/scene/material_map/transform.md")]
    pub fn transform_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] offset: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] scale: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] rotation: &JsValue,
    ) -> Result<Self, WebError> {
        let offset = read_vector::<2>(offset, "MaterialMap.transform: offset")?;
        let scale = read_vector::<2>(scale, "MaterialMap.transform: scale")?;
        let rotation = read_finite(rotation, "MaterialMap.transform: rotation")?;
        Ok(self.transform(offset, scale, rotation)?)
    }
}

/// Reads another map, which takes no options, or any input `createTexture` takes.
fn read_input(
    input: &JsValue,
    options: Option<&JsValue>,
    call: &str,
) -> Result<TextureInput, WebError> {
    let Ok(map) = MaterialMap::try_from(input) else {
        return read_texture_input(input, options, call);
    };
    match options.filter(|options| !options.is_undefined() && !options.is_null()) {
        Some(_) => Err(refuse_options(call).into()),
        None => Ok(TextureData::Map(map).into()),
    }
}
