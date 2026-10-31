#![cfg(wasm)]

use crate::error::WebError;
use js_sys::{
    Array, ArrayBuffer, DataView, Float32Array, Int32Array, Object, Reflect, Uint8Array,
    Uint32Array,
};
use lsp_doc::lsp_doc;
use std::convert::TryInto;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::bindings::web::{describe, read_bytes, read_class, read_numbers, read_rows, read_text};
use crate::core::shader::UniformInput;
use crate::{Shader, ShaderError, UniformData};

#[wasm_bindgen]
impl Shader {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new_js(input: &JsValue) -> Result<Shader, WebError> {
        Ok(Shader::new(source_parts(input)?)?)
    }

    #[wasm_bindgen(js_name = "setRegistry")]
    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry_js(
        #[wasm_bindgen(unchecked_param_type = "string")] base_url: &JsValue,
    ) -> Result<(), WebError> {
        Shader::set_registry(&read_text(base_url, "Shader.setRegistry: base_url")?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "fetch")]
    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub async fn fetch_js(input: &JsValue) -> Result<Shader, WebError> {
        Ok(Shader::fetch(source_parts(input)?).await?)
    }

    // Static helpers that mirror Rust API for the JS side
    #[wasm_bindgen(js_name = "fromVertex")]
    #[lsp_doc("docs/api/core/shader/from_vertex.md")]
    pub fn from_vertex_js(v: &crate::mesh::Vertex) -> Result<Shader, ShaderError> {
        Self::from_vertex(v)
    }

    #[wasm_bindgen(js_name = "fromMesh")]
    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh_js(m: &crate::mesh::Mesh) -> Result<Shader, ShaderError> {
        Self::from_mesh(m)
    }

    #[wasm_bindgen(js_name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "string")] key: &JsValue,
        value: &JsValue,
    ) -> Result<(), WebError> {
        let key = read_text(key, "Shader.set: key")?;
        Ok(self.object.set_input(&key, uniform_input(&key, value)?)?)
    }

    #[wasm_bindgen(js_name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "string")] key: &JsValue,
    ) -> Result<JsValue, WebError> {
        let key = read_text(key, "Shader.get: key")?;
        Ok(uniform_value(self.object.get_uniform_data(&key)?))
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
    pub fn remove_meshes_js(&self, list: &JsValue) -> Result<(), WebError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| WebError::type_error("removeMeshes: expected an array of Mesh"))?;
        let mut meshes: Vec<crate::mesh::Mesh> = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            let m: crate::mesh::Mesh = (&v)
                .try_into()
                .map_err(|_| WebError::type_error("removeMeshes: item is not a Mesh"))?;
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
/// A `Uint8Array`, `ArrayBuffer` or `DataView` is raw bytes. Any other typed array or plain
/// array is numbers, and an array of number arrays, such as matrix columns, is flattened.
/// Any other value is a `TypeError`, as Python raises. A value of the right JS type that the
/// declared WGSL type can't hold fails later, in `set_input`.
fn uniform_input(key: &str, value: &JsValue) -> Result<UniformInput, WebError> {
    if let Some(number) = value.as_f64() {
        return Ok(UniformInput::Number(number));
    }
    if let Some(flag) = value.as_bool() {
        return Ok(UniformInput::Bool(flag));
    }
    let raw = value.is_instance_of::<Uint8Array>()
        || value.is_instance_of::<ArrayBuffer>()
        || value.is_instance_of::<DataView>();
    if raw && let Some(bytes) = read_bytes(value) {
        return Ok(UniformInput::Bytes(bytes));
    }
    if let Some(numbers) = read_numbers(value) {
        return Ok(UniformInput::Numbers(numbers));
    }
    if let Some(rows) = read_rows(value) {
        return Ok(UniformInput::Numbers(rows.concat()));
    }
    if Array::is_array(value) {
        return Err(WebError::type_error(&format!(
            "Shader.set: {key}: expected an array of numbers, or an array of number arrays"
        )));
    }
    // Exported classes carry `__wbg_ptr`. Texture is the only one a uniform
    // accepts, and its bridge checks the brand before it reads the pointer.
    if let Some(kind) = read_class(value) {
        if let Ok(texture) = crate::texture::Texture::try_from(value) {
            return Ok(UniformInput::Texture(*texture.id()));
        }
        return Err(WebError::type_error(&format!(
            "Shader.set: {key}: expected a Texture, got {kind}"
        )));
    }
    let kind = describe(value);
    Err(WebError::type_error(&format!(
        "Shader.set: {key}: expected a number, an array, a typed array, bytes, or a Texture, got {kind}"
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

/// Reads a shader source: one string, or an array of strings to compose.
/// Any other value is a `TypeError`, as Python raises.
fn source_parts(input: &JsValue) -> Result<Vec<String>, WebError> {
    if let Some(source) = input.as_string() {
        return Ok(vec![source]);
    }
    let Some(array) = input.dyn_ref::<Array>() else {
        return Err(WebError::type_error(&format!(
            "Shader input must be a string or an array of strings, got {}",
            describe(input)
        )));
    };
    array
        .iter()
        .map(|item| item.as_string())
        .collect::<Option<Vec<String>>>()
        .ok_or_else(|| WebError::type_error("Shader input array contains a non-string element"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    // Story: JS values sort into loose inputs without guessing a WGSL type.
    #[wasm_bindgen_test]
    fn uniform_input_sorts_js_values() {
        let input = |value: JsValue| uniform_input("key", &value).ok();
        assert_eq!(
            input(JsValue::from_f64(42.0)),
            Some(UniformInput::Number(42.0))
        );
        assert_eq!(
            input(JsValue::from_bool(true)),
            Some(UniformInput::Bool(true))
        );

        let array = Array::of2(&JsValue::from_f64(1.0), &JsValue::from_f64(2.0));
        let numbers = UniformInput::Numbers(vec![1.0, 2.0]);
        assert_eq!(input(array.into()), Some(numbers));
        let ints = Int32Array::from(&[3, -4][..]);
        let numbers = UniformInput::Numbers(vec![3.0, -4.0]);
        assert_eq!(input(ints.into()), Some(numbers));
        let bytes = Uint8Array::from(&[1u8, 2, 3][..]);
        assert_eq!(
            input(bytes.into()),
            Some(UniformInput::Bytes(vec![1, 2, 3]))
        );

        let columns = Array::of2(
            &Array::of2(&JsValue::from_f64(1.0), &JsValue::from_f64(0.0)),
            &Float32Array::from(&[0.0f32, 1.0][..]),
        );
        let identity = UniformInput::Numbers(vec![1.0, 0.0, 0.0, 1.0]);
        assert_eq!(input(columns.into()), Some(identity));
        let buffer = Uint8Array::from(&[4u8, 5][..]).buffer();
        assert_eq!(input(buffer.into()), Some(UniformInput::Bytes(vec![4, 5])));

        assert!(uniform_input("key", &JsValue::from_str("1.0")).is_err());
        let mixed = Array::of2(&JsValue::from_f64(1.0), &JsValue::from_str("x"));
        assert!(uniform_input("key", &mixed.into()).is_err());
    }
}
