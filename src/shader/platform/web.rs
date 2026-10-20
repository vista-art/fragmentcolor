#![cfg(wasm)]

use js_sys::{
    Array, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Object, Reflect,
    Uint8Array, Uint8ClampedArray, Uint16Array, Uint32Array,
};
use lsp_doc::lsp_doc;
use std::convert::TryInto;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::shader::UniformInput;
use crate::{Shader, ShaderError, UniformData};

#[wasm_bindgen]
impl Shader {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new_js(input: &JsValue) -> Self {
        let parts = match jsvalue_to_string_vec(input) {
            Ok(v) => v,
            Err(msg) => {
                console::error_1(&msg.into());
                return Shader::default();
            }
        };
        match Shader::new(parts) {
            Ok(shader) => shader,
            Err(_) => {
                console::error_1(&"failed to create shader, returning default".into());
                Shader::default()
            }
        }
    }

    #[wasm_bindgen(js_name = "setRegistry")]
    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry_js(base_url: &str) {
        Shader::set_registry(base_url);
    }

    #[wasm_bindgen(js_name = "fetch")]
    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub async fn fetch_js(input: &JsValue) -> Self {
        let parts = match jsvalue_to_string_vec(input) {
            Ok(v) => v,
            Err(msg) => {
                console::error_1(&msg.into());
                return Shader::default();
            }
        };

        match Shader::fetch(parts).await {
            Ok(shader) => shader,
            Err(e) => {
                console::error_1(&e.to_string().into());
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
        self.object.set_input(key, uniform_input(key, value)?)
    }

    #[wasm_bindgen(js_name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_js(&self, key: &str) -> Result<JsValue, ShaderError> {
        Ok(uniform_value(self.object.get_uniform_data(key)?))
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
    #[doc(hidden)]
    #[wasm_bindgen(js_name = "unwrap")]
    pub fn unwrap_js(&self) -> Self {
        self.clone()
    }

    #[wasm_bindgen(js_name = "default")]
    #[lsp_doc("docs/api/core/shader/default.md")]
    pub fn default_js() -> Self {
        Shader::default()
    }
}

/// Sorts a JS value into a loose uniform input. The key's declared WGSL type does the rest.
/// A `Uint8Array` is raw bytes, and any other typed array or plain array is numbers.
fn uniform_input(key: &str, value: &JsValue) -> Result<UniformInput, ShaderError> {
    if let Some(number) = value.as_f64() {
        return Ok(UniformInput::Number(number));
    }
    if let Some(flag) = value.as_bool() {
        return Ok(UniformInput::Bool(flag));
    }
    if let Some(bytes) = value.dyn_ref::<Uint8Array>() {
        return Ok(UniformInput::Bytes(bytes.to_vec()));
    }
    if let Some(array) = value.dyn_ref::<Array>() {
        let numbers = array.iter().map(|item| item.as_f64());
        return match numbers.collect::<Option<Vec<f64>>>() {
            Some(numbers) => Ok(UniformInput::Numbers(numbers)),
            None => Err(ShaderError::TypeMismatch(format!(
                "{key}: expected an array of numbers"
            ))),
        };
    }
    macro_rules! typed_arrays {
        ($($kind:ty),*) => {$(
            if let Some(array) = value.dyn_ref::<$kind>() {
                let numbers = array.to_vec().into_iter().map(f64::from).collect();
                return Ok(UniformInput::Numbers(numbers));
            }
        )*};
    }
    typed_arrays!(
        Float32Array,
        Float64Array,
        Int32Array,
        Uint32Array,
        Int16Array,
        Uint16Array,
        Int8Array,
        Uint8ClampedArray
    );
    // Exported classes carry `__wbg_ptr`. Texture is the only one a uniform
    // accepts, and its bridge checks the brand before it reads the pointer.
    if value.is_object() && Reflect::has(value, &"__wbg_ptr".into()).unwrap_or(false) {
        if let Ok(texture) = crate::texture::Texture::try_from(value) {
            return Ok(UniformInput::Texture(*texture.id()));
        }
        let kind = Reflect::get(value, &"__fc_kind".into())
            .ok()
            .and_then(|kind| kind.as_string())
            .unwrap_or_else(|| "an unknown object".into());
        return Err(ShaderError::TypeMismatch(format!(
            "{key}: expected a Texture, got {kind}"
        )));
    }
    let kind = value.js_typeof().as_string().unwrap_or_default();
    Err(ShaderError::TypeMismatch(format!(
        "{key}: expected a number, an array, a typed array, or a Texture, got {kind}"
    )))
}

/// Turns the value `Shader.get` read into JS: numbers, typed arrays, a texture id, or bytes.
fn uniform_value(data: UniformData) -> JsValue {
    use UniformData as U;
    match data {
        U::Bool(flag) => JsValue::from_bool(flag),
        U::Float(v) => JsValue::from_f64(v.into()),
        U::Int(v) => JsValue::from_f64(v.into()),
        U::UInt(v) => JsValue::from_f64(v.into()),
        U::Vec2(v) | U::Vec3(v) | U::Vec4(v) | U::Mat2(v) | U::Mat3(v) | U::Mat4(v) => {
            Float32Array::from(&v[..]).into()
        }
        U::IVec2(v) | U::IVec3(v) | U::IVec4(v) => Int32Array::from(&v[..]).into(),
        U::UVec2(v) | U::UVec3(v) | U::UVec4(v) => Uint32Array::from(&v[..]).into(),
        U::Texture(meta) => JsValue::from_f64(meta.id.id as f64),
        U::Sampler(info) => {
            let object = Object::new();
            let comparison = JsValue::from_bool(info.comparison);
            let _ = Reflect::set(&object, &"comparison".into(), &comparison);
            object.into()
        }
        other => Uint8Array::from(&other.to_bytes()[..]).into(),
    }
}

/// Accept either a single JS string or a JS array of strings, return Vec<String>.
fn jsvalue_to_string_vec(input: &JsValue) -> Result<Vec<String>, String> {
    if let Some(s) = input.as_string() {
        return Ok(vec![s]);
    }
    if js_sys::Array::is_array(input) {
        let arr = js_sys::Array::from(input);
        let mut out = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            match v.as_string() {
                Some(s) => out.push(s),
                None => return Err("Shader input array contains a non-string element".into()),
            }
        }
        return Ok(out);
    }
    Err("Shader input must be a string or an array of strings".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // Story: JS values sort into loose inputs without guessing a WGSL type.
    #[wasm_bindgen_test]
    fn uniform_input_sorts_js_values() {
        let input = |value: JsValue| uniform_input("key", &value).expect("supported value");
        assert_eq!(input(JsValue::from_f64(42.0)), UniformInput::Number(42.0));
        assert_eq!(input(JsValue::from_bool(true)), UniformInput::Bool(true));

        let array = Array::of2(&JsValue::from_f64(1.0), &JsValue::from_f64(2.0));
        assert_eq!(input(array.into()), UniformInput::Numbers(vec![1.0, 2.0]));
        let ints = Int32Array::from(&[3, -4][..]);
        assert_eq!(input(ints.into()), UniformInput::Numbers(vec![3.0, -4.0]));
        let bytes = Uint8Array::from(&[1u8, 2, 3][..]);
        assert_eq!(input(bytes.into()), UniformInput::Bytes(vec![1, 2, 3]));

        assert!(uniform_input("key", &JsValue::from_str("1.0")).is_err());
        let mixed = Array::of2(&JsValue::from_f64(1.0), &JsValue::from_str("x"));
        assert!(uniform_input("key", &mixed.into()).is_err());
    }
}
