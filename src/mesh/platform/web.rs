#![cfg(wasm)]

use crate::error::WebError;
use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::mesh::{Instance, Mesh, Quad, Vertex, VertexData};
use crate::platforms::vector;
use crate::platforms::web::{
    read_count, read_floats, read_numbers, read_text, read_unsigned, read_vector,
};

// -----------------------------
// JS helpers
// -----------------------------
/// Reads a vertex attribute value: a number,
/// or an array or typed array of 1 to 4 numbers.
/// They take the type the shader declares when the mesh attaches to it.
fn read_attribute(value: &JsValue) -> Result<VertexData, WebError> {
    if let Some(number) = value.as_f64() {
        return Ok(number.into());
    }
    let numbers = read_numbers(value).ok_or_else(|| {
        WebError::type_error("a vertex attribute is a number or a list of 1 to 4 numbers")
    })?;
    Ok(VertexData::try_from(numbers.as_slice())?)
}

/// Reads a vertex position: a `Vertex`, a number, or an array or `Float32Array`
/// of 1 to 4 numbers, each checked by the float vector rule under `label`.
fn read_position(value: &JsValue, label: &str) -> Result<Vertex, WebError> {
    if let Ok(vertex) = Vertex::try_from(value) {
        return Ok(vertex);
    }
    let vertex = match value.as_f64() {
        Some(number) => vector::read_position([number], label)?,
        None => vector::read_position(read_floats(value, label)?, label)?,
    };
    Ok(vertex)
}

/// Reads a vertex for the `Mesh` methods: anything `new Vertex` takes, or the
/// `[position, uv]` and `[position, uv, color]` forms of Rust's `Into<Vertex>`,
/// whose parts take exactly 3, 2 and 4 numbers under the float vector rule, each named after `label`.
fn read_vertex(value: &JsValue, label: &str) -> Result<Vertex, WebError> {
    let parts = match value.dyn_ref::<js_sys::Array>() {
        Some(array) if array.length() > 1 && array.iter().all(|part| part.is_object()) => {
            array.to_vec()
        }
        _ => return read_position(value, label),
    };
    let position = |part: &JsValue| read_vector::<3>(part, &format!("{label}: position"));
    let uv = |part: &JsValue| read_vector::<2>(part, &format!("{label}: uv"));
    let color = |part: &JsValue| read_vector::<4>(part, &format!("{label}: color"));
    match parts.as_slice() {
        [first, second] => Ok(Vertex::from((position(first)?, uv(second)?))),
        [first, second, third] => Ok(Vertex::from((position(first)?, uv(second)?, color(third)?))),
        _ => Err(WebError::type_error(&format!(
            "{label} takes [position, uv] or [position, uv, color] in parts, got {} parts",
            parts.len()
        ))),
    }
}

fn js_to_instance_into(value: &JsValue) -> Result<Instance, WebError> {
    // 1. Already an Instance JS object?
    if let Ok(instance) = Instance::try_from(value) {
        return Ok(instance);
    }
    // 2. Vertex object (backward-compat: convert via create_instance, dropping position).
    if let Ok(vertex) = Vertex::try_from(value) {
        return Ok(vertex.create_instance());
    }
    // 3. Plain JS object: { key: value, ... } → Instance::new().set(...).set(...)
    if value.is_object() && !js_sys::Array::is_array(value) {
        let obj = js_sys::Object::from(value.clone());
        let entries = js_sys::Object::entries(&obj);
        let mut instance = Instance::new();
        for entry in entries.iter() {
            let pair = js_sys::Array::from(&entry);
            let key = pair
                .get(0)
                .as_string()
                .ok_or_else(|| WebError::type_error("Instance attribute keys must be strings"))?;
            instance = instance.set(&key, read_attribute(&pair.get(1))?);
        }
        return Ok(instance);
    }
    Err(WebError::type_error(
        "Expected an Instance, a Vertex, or an object of named attributes",
    ))
}

// -----------------------------
// JS conversions for Vertex/Instance owned via __wbg_ptr anchors
// -----------------------------

crate::impl_js_bridge!(Mesh, crate::mesh::error::MeshError);
crate::impl_js_bridge!(Quad, crate::mesh::error::MeshError);
crate::impl_js_bridge!(Vertex, crate::mesh::error::MeshError);
crate::impl_js_bridge!(Instance, crate::mesh::error::MeshError);

// -----------------------------
// Vertex (WASM bindings)
// -----------------------------
#[wasm_bindgen]
impl Vertex {
    // The attribute names `Vertex::NORMAL` and friends, as static getters on the
    // JS class. Python exposes the same constants as class attributes.
    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "NORMAL")]
    pub fn normal_js() -> String {
        Self::NORMAL.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "TANGENT")]
    pub fn tangent_js() -> String {
        Self::TANGENT.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "UV0")]
    pub fn uv0_js() -> String {
        Self::UV0.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "UV1")]
    pub fn uv1_js() -> String {
        Self::UV1.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "COLOR0")]
    pub fn color0_js() -> String {
        Self::COLOR0.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "COLOR1")]
    pub fn color1_js() -> String {
        Self::COLOR1.into()
    }

    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/vertex/new.md")]
    pub fn new_js(position: &JsValue) -> Result<Vertex, WebError> {
        read_position(position, "Vertex: position")
    }

    // Support Rust-style Vertex::new([...]) in JS as Vertex.new([...])
    // JS compat shim — same entry point as the constructor; not separately documented.
    #[doc(hidden)]
    #[wasm_bindgen(js_name = "new")]
    pub fn new_static(position: &JsValue) -> Result<Vertex, WebError> {
        read_position(position, "Vertex: position")
    }

    #[wasm_bindgen(js_name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_js(position: &JsValue) -> Result<Vertex, WebError> {
        let v = read_position(position, "Vertex.pbr: position")?;
        Ok(v.set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0]))
    }

    #[wasm_bindgen(js_name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "string")] key: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number | ArrayLike<number>")] value: &JsValue,
    ) -> Result<Vertex, WebError> {
        let key = read_text(key, "Vertex.set: key")?;
        Ok(self.clone().set(&key, read_attribute(value)?))
    }

    #[wasm_bindgen(js_name = "createInstance")]
    #[lsp_doc("docs/api/geometry/vertex/create_instance.md")]
    pub fn create_instance_js(&self) -> Instance {
        self.create_instance()
    }
}

// -----------------------------
// Instance (WASM bindings): `new Instance()` + `.set(key, value)` builder.
// `mesh.addInstance` also takes a plain `{ key: value, ... }` object.
// -----------------------------
#[wasm_bindgen]
impl Instance {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/instance/new.md")]
    pub fn new_js() -> Instance {
        Instance::new()
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "new")]
    pub fn new_static() -> Instance {
        Instance::new()
    }

    #[wasm_bindgen(js_name = "set")]
    #[lsp_doc("docs/api/geometry/instance/set.md")]
    pub fn set_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "string")] key: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "number | ArrayLike<number>")] value: &JsValue,
    ) -> Result<Instance, WebError> {
        let key = read_text(key, "Instance.set: key")?;
        Ok(self.clone().set(&key, read_attribute(value)?))
    }
}

// -----------------------------
// Mesh (WASM bindings)
// -----------------------------
#[wasm_bindgen]
impl Mesh {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/mesh/new.md")]
    pub fn new_js() -> Mesh {
        Mesh::new()
    }

    #[wasm_bindgen(js_name = "fromVertices")]
    #[lsp_doc("docs/api/geometry/mesh/from_vertices.md")]
    pub fn from_vertices_js(list: &JsValue) -> Result<Mesh, WebError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| WebError::type_error("Expected an array"))?;
        let mut vertices = Vec::with_capacity(arr.length() as usize);
        for (index, v) in arr.iter().enumerate() {
            vertices.push(read_vertex(
                &v,
                &format!("Mesh.fromVertices: vertex {index}"),
            )?);
        }
        Ok(Mesh::from_vertices(vertices))
    }

    #[wasm_bindgen(js_name = "addVertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_js(&mut self, v: &JsValue) -> Result<(), WebError> {
        let vertex = read_vertex(v, "Mesh.addVertex: vertex")?;
        self.add_vertex(vertex);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addVertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_js(&mut self, list: &JsValue) -> Result<(), WebError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| WebError::type_error("Expected an array"))?;
        let mut verts: Vec<Vertex> = Vec::with_capacity(arr.length() as usize);
        for (index, v) in arr.iter().enumerate() {
            let vertex = read_vertex(&v, &format!("Mesh.addVertices: vertex {index}"))?;
            verts.push(vertex);
        }
        self.add_vertices(verts);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addInstance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_js(&mut self, item: &JsValue) -> Result<(), WebError> {
        let inst = js_to_instance_into(item)?;
        self.add_instance(inst);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addInstances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_js(&mut self, list: &JsValue) -> Result<(), WebError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| WebError::type_error("Expected an array"))?;
        let mut instances: Vec<Instance> = Vec::with_capacity(arr.length() as usize);
        for instance in arr.iter() {
            let instance = js_to_instance_into(&instance)?;
            instances.push(instance);
        }
        self.add_instances(instances);
        Ok(())
    }

    #[wasm_bindgen(js_name = "clearInstances")]
    #[lsp_doc("docs/api/geometry/mesh/clear_instances.md")]
    pub fn clear_instances_js(&mut self) {
        self.clear_instances();
    }

    #[wasm_bindgen(js_name = "setInstanceCount")]
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "number")] count: &JsValue,
    ) -> Result<(), WebError> {
        self.set_instance_count(read_count(count, "Mesh.setInstanceCount: count")?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "setIndices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] indices: &JsValue,
    ) -> Result<(), WebError> {
        if let Some(array) = indices.dyn_ref::<js_sys::Uint32Array>() {
            self.set_indices(array.to_vec());
            return Ok(());
        }
        let numbers = read_numbers(indices).ok_or_else(|| {
            WebError::type_error("Mesh.setIndices: expected an array or typed array of numbers")
        })?;
        let read = |(index, number): (usize, f64)| {
            read_unsigned(number, &format!("Mesh.setIndices: item {index}"))
        };
        let indices = numbers.into_iter().enumerate().map(read);
        self.set_indices(indices.collect::<Result<Vec<u32>, WebError>>()?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "clearIndices")]
    #[lsp_doc("docs/api/geometry/mesh/clear_indices.md")]
    pub fn clear_indices_js(&mut self) {
        self.clear_indices();
    }
}

// -----------------------------
// Quad (WASM bindings)
// -----------------------------
#[wasm_bindgen]
impl Quad {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/quad/new.md")]
    pub fn new_js(min: &JsValue, max: &JsValue) -> Result<Quad, WebError> {
        Ok(Quad::new(
            read_vector::<2>(min, "Quad: min")?,
            read_vector::<2>(max, "Quad: max")?,
        ))
    }

    #[wasm_bindgen(js_name = "getMesh")]
    #[lsp_doc("docs/api/geometry/quad/get_mesh.md")]
    pub fn get_mesh_js(&self) -> Mesh {
        self.get_mesh()
    }
}
