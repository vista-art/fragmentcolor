#![cfg(wasm)]

use lsp_doc::lsp_doc;
use std::convert::TryInto;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, future_to_promise};
use web_sys::{Request, RequestInit, RequestMode, Response, console};

use crate::{Shader, ShaderError, UniformData};

#[wasm_bindgen]
impl Shader {
    #[wasm_bindgen(constructor)]
    pub fn new_js(source: &str) -> Self {
        if let Ok(shader) = Shader::new(source) {
            shader
        } else {
            console::error_1(&"failed to create shader".into());
            Shader::new(crate::constants::DEFAULT_SHADER).expect("failed to create default shader")
        }
    }

#[lsp_doc("docs/api/core/shader/fetch.md")]
    pub async fn fetch(url: &str) -> Self {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts).expect("failed to create request");
        let window = web_sys::window().expect("no global `window` exists");
        let resp_promise = window.fetch_with_request(&request);
        let resp_value = future_to_promise(JsFuture::from(resp_promise));

        let resp: Response = resp_value.dyn_into().expect("not a Response");

        let jsvalue = JsFuture::from(resp.text().expect("failed to read response"))
            .await
            .expect("failed to read response");

        let body = jsvalue.as_string().expect("response not a string");

        Self::new_js(&body)
    }

    #[wasm_bindgen(js_name = "set")]
#[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_js(&self, key: &str, value: JsValue) -> Result<(), ShaderError> {
        let uniform_data: UniformData = value.try_into()?;
        self.object.set(key, uniform_data)
    }

    #[wasm_bindgen(js_name = "get")]
#[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_js(&self, key: &str) -> Result<JsValue, ShaderError> {
        let uniform_data = self.object.get_uniform_data(key)?;
        Ok(uniform_data.into())
    }

    #[wasm_bindgen(js_name = "listUniforms")]
#[lsp_doc("docs/api/core/shader/list_uniforms.md")]
    pub fn list_uniforms_js(&self) -> Vec<String> {
        self.list_uniforms()
    }

    #[wasm_bindgen(js_name = "listKeys")]
#[lsp_doc("docs/api/core/shader/list_keys.md")]
    pub fn list_keys_js(&self) -> Vec<String> {
        self.list_keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::uniform::UniformData;
    use js_sys::{Array, Float32Array};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_uniform_data_try_from_jsvalue() {
        use std::convert::TryInto;

        // Test boolean
        let js_bool = JsValue::from_bool(true);
        let data: UniformData = js_bool.try_into().unwrap();
        match data {
            UniformData::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected Bool"),
        }

        // Test float
        let js_float = JsValue::from_f64(3.14);
        let data: UniformData = js_float.try_into().unwrap();
        match data {
            UniformData::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }

        // Test integer
        let js_int = JsValue::from_f64(42.0);
        let data: UniformData = js_int.try_into().unwrap();
        match data {
            UniformData::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int"),
        }

        // Test array as vec2
        let array = Array::new();
        array.push(&JsValue::from_f64(1.0));
        array.push(&JsValue::from_f64(2.0));
        let js_array: JsValue = array.into();
        let data: UniformData = js_array.try_into().unwrap();
        match data {
            UniformData::Vec2(v) => assert_eq!(v, [1.0, 2.0]),
            _ => panic!("Expected Vec2"),
        }

        // Test array as vec3
        let array = Array::new();
        array.push(&JsValue::from_f64(1.0));
        array.push(&JsValue::from_f64(2.0));
        array.push(&JsValue::from_f64(3.0));
        let js_array: JsValue = array.into();
        let data: UniformData = js_array.try_into().unwrap();
        match data {
            UniformData::Vec3(v) => assert_eq!(v, [1.0, 2.0, 3.0]),
            _ => panic!("Expected Vec3"),
        }

        // Test array as vec4
        let array = Array::new();
        array.push(&JsValue::from_f64(1.0));
        array.push(&JsValue::from_f64(2.0));
        array.push(&JsValue::from_f64(3.0));
        array.push(&JsValue::from_f64(4.0));
        let js_array: JsValue = array.into();
        let data: UniformData = js_array.try_into().unwrap();
        match data {
            UniformData::Vec4(v) => assert_eq!(v, [1.0, 2.0, 3.0, 4.0]),
            _ => panic!("Expected Vec4"),
        }

        // Test typed array
        let f32_array = Float32Array::new_with_length(3);
        f32_array.set_index(0, 1.0);
        f32_array.set_index(1, 2.0);
        f32_array.set_index(2, 3.0);
        let js_array: JsValue = f32_array.into();
        let data: UniformData = js_array.try_into().unwrap();
        match data {
            UniformData::Vec3(v) => assert_eq!(v, [1.0, 2.0, 3.0]),
            _ => panic!("Expected Vec3 from Float32Array"),
        }
    }
}
