#![cfg(wasm)]

use lsp_doc::lsp_doc;
use std::convert::TryInto;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::{Shader, ShaderError, UniformData};

#[wasm_bindgen]
impl Shader {
    #[wasm_bindgen(constructor)]
    pub fn new_js(source: &str) -> Self {
        if let Ok(shader) = Shader::new(source) {
            shader
        } else {
            console::error_1(&"failed to create shader, returning default".into());
            Shader::default()
        }
    }

    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub async fn fetch(url: &str) -> Self {
        match crate::net::fetch_text(url).await {
            Ok(body) => Self::new_js(&body),
            Err(e) => {
                console::error_1(&e);
                Shader::default()
            }
        }
    }

    // Static helpers that mirror Rust API for the JS side
    #[wasm_bindgen(js_name = "fromVertex")]
    #[lsp_doc("docs/api/core/shader/from_vertex.md")]
    pub fn from_vertex_js(v: &crate::mesh::Vertex) -> Self {
        Self::from_vertex(v)
    }

    #[wasm_bindgen(js_name = "fromMesh")]
    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh_js(m: &crate::mesh::Mesh) -> Self {
        Self::from_mesh(m)
    }

    #[wasm_bindgen(js_name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_js(&self, key: &str, value: &JsValue) -> Result<(), ShaderError> {
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

    #[wasm_bindgen(js_name = "addMesh")]
    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh_js(&self, mesh: &crate::mesh::Mesh) -> Result<(), ShaderError> {
        self.add_mesh(mesh)
    }

    #[wasm_bindgen(js_name = "removeMesh")]
    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh_js(&self, mesh: &crate::mesh::Mesh) {
        self.remove_mesh(mesh)
    }

    #[wasm_bindgen(js_name = "removeMeshes")]
    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes_js(&self, list: &JsValue) -> Result<(), JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("removeMeshes: expected an array of Mesh"))?;
        let mut meshes: Vec<crate::mesh::Mesh> = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            let m: crate::mesh::Mesh = (&v)
                .try_into()
                .map_err(|_| JsError::new("removeMeshes: item is not a Mesh"))?;
            meshes.push(m);
        }
        for m in meshes.iter() {
            self.remove_mesh(m);
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "clearMeshes")]
    #[lsp_doc("docs/api/core/shader/clear_meshes.md")]
    pub fn clear_meshes_js(&self) {
        self.clear_meshes()
    }

    // Instance helpers mirrored to JS
    #[wasm_bindgen(js_name = "validateMesh")]
    #[lsp_doc("docs/api/core/shader/validate_mesh.md")]
    pub fn validate_mesh_js(&self, mesh: &crate::mesh::Mesh) -> Result<(), ShaderError> {
        self.validate_mesh(mesh)
    }

    #[wasm_bindgen(js_name = "isCompute")]
    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute_js(&self) -> bool {
        self.is_compute()
    }

    // No-op Result::unwrap shim for transpiled examples that used Rust .unwrap()
    #[wasm_bindgen(js_name = "unwrap")]
    pub fn unwrap_js(&self) -> Self {
        self.clone()
    }

    #[wasm_bindgen(js_name = "default")]
    pub fn default_js() -> Self {
        Shader::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::uniform::UniformData;
    // use js_sys::{Array, Float32Array};
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_uniform_data_try_from_jsvalue() {
        use std::convert::TryInto;

        // Test boolean
        let js_bool = &JsValue::from_bool(true);
        let data: UniformData = js_bool.try_into().unwrap();
        match data {
            UniformData::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected Bool"),
        }

        // Test float
        let js_float = &JsValue::from_f64(3.14);
        let data: UniformData = js_float.try_into().unwrap();
        match data {
            UniformData::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }

        // Test integer
        let js_int = &JsValue::from_f64(42.0);
        let data: UniformData = js_int.try_into().unwrap();
        match data {
            UniformData::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int"),
        }

        // @TODO uncomment and fix
        //
        // Test array as vec2
        let array = &js_sys::Array::new();
        array.push(&JsValue::from_f64(1.0));
        array.push(&JsValue::from_f64(2.0));
        let js_array: JsValue = array.into();
        let data: UniformData = js_array.try_into().unwrap();
        match data {
            UniformData::Vec2(v) => assert_eq!(v, [1.0, 2.0]),
            _ => panic!("Expected Vec2"),
        }

        // // Test array as vec3
        // let array = Array::new();
        // array.push(&JsValue::from_f64(1.0));
        // array.push(&JsValue::from_f64(2.0));
        // array.push(&JsValue::from_f64(3.0));
        // let js_array: JsValue = array.into();
        // let data: UniformData = js_array.try_into().unwrap();
        // match data {
        //     UniformData::Vec3(v) => assert_eq!(v, [1.0, 2.0, 3.0]),
        //     _ => panic!("Expected Vec3"),
        // }

        // // Test array as vec4
        // let array = Array::new();
        // array.push(&JsValue::from_f64(1.0));
        // array.push(&JsValue::from_f64(2.0));
        // array.push(&JsValue::from_f64(3.0));
        // array.push(&JsValue::from_f64(4.0));
        // let js_array: JsValue = array.into();
        // let data: UniformData = js_array.try_into().unwrap();
        // match data {
        //     UniformData::Vec4(v) => assert_eq!(v, [1.0, 2.0, 3.0, 4.0]),
        //     _ => panic!("Expected Vec4"),
        // }

        // // Test typed array
        // let f32_array = Float32Array::new_with_length(3);
        // f32_array.set_index(0, 1.0);
        // f32_array.set_index(1, 2.0);
        // f32_array.set_index(2, 3.0);
        // let js_array: JsValue = f32_array.into();
        // let data: UniformData = js_array.try_into().unwrap();
        // match data {
        //     UniformData::Vec3(v) => assert_eq!(v, [1.0, 2.0, 3.0]),
        //     _ => panic!("Expected Vec3 from Float32Array"),
        // }
    }
}
