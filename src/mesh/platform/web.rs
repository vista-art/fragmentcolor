#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::mesh::{Instance, Mesh, Quad, Vertex, VertexValue};

// -----------------------------
// JS helpers
// -----------------------------
fn js_to_f32_vec(value: &JsValue) -> Option<Vec<f32>> {
    use js_sys::{Array, Float32Array, Int32Array, Uint32Array};

    if let Some(arr) = value.dyn_ref::<Float32Array>() {
        let len = arr.length() as usize;
        let mut buf = vec![0.0f32; len];
        arr.copy_to(&mut buf[..]);
        return Some(buf);
    }
    if let Some(arr) = value.dyn_ref::<Int32Array>() {
        let len = arr.length() as usize;
        let mut buf_i = vec![0i32; len];
        arr.copy_to(&mut buf_i[..]);
        return Some(buf_i.into_iter().map(|v| v as f32).collect());
    }
    if let Some(arr) = value.dyn_ref::<Uint32Array>() {
        let len = arr.length() as usize;
        let mut buf_u = vec![0u32; len];
        arr.copy_to(&mut buf_u[..]);
        return Some(buf_u.into_iter().map(|v| v as f32).collect());
    }
    if let Some(arr) = value.dyn_ref::<Array>() {
        let len = arr.length() as usize;
        let mut out = Vec::with_capacity(len);
        for i in 0..(len as u32) {
            let v = arr.get(i).as_f64()? as f32;
            out.push(v);
        }
        return Some(out);
    }
    if let Some(n) = value.as_f64() {
        return Some(vec![n as f32]);
    }
    None
}

fn js_to_vertex_value(value: &JsValue) -> Result<VertexValue, JsError> {
    use js_sys::{Float32Array, Int32Array, Uint32Array};

    // Float32Array
    if let Some(arr) = value.dyn_ref::<Float32Array>() {
        let len = arr.length();
        match len {
            1 => {
                let mut v = [0.0f32; 1];
                arr.copy_to(&mut v);
                return Ok(VertexValue::F32(v[0]));
            }
            2 => {
                let mut v = [0.0f32; 2];
                arr.copy_to(&mut v);
                return Ok(VertexValue::F32x2(v));
            }
            3 => {
                let mut v = [0.0f32; 3];
                arr.copy_to(&mut v);
                return Ok(VertexValue::F32x3(v));
            }
            4 => {
                let mut v = [0.0f32; 4];
                arr.copy_to(&mut v);
                return Ok(VertexValue::F32x4(v));
            }
            _ => {
                return Err(JsError::new(
                    "Unsupported Float32Array length for vertex attribute",
                ));
            }
        }
    }
    // Int32Array
    if let Some(arr) = value.dyn_ref::<Int32Array>() {
        let len = arr.length();
        match len {
            1 => {
                let mut v = [0i32; 1];
                arr.copy_to(&mut v);
                return Ok(VertexValue::I32(v[0]));
            }
            2 => {
                let mut v = [0i32; 2];
                arr.copy_to(&mut v);
                return Ok(VertexValue::I32x2(v));
            }
            3 => {
                let mut v = [0i32; 3];
                arr.copy_to(&mut v);
                return Ok(VertexValue::I32x3(v));
            }
            4 => {
                let mut v = [0i32; 4];
                arr.copy_to(&mut v);
                return Ok(VertexValue::I32x4(v));
            }
            _ => {
                return Err(JsError::new(
                    "Unsupported Int32Array length for vertex attribute",
                ));
            }
        }
    }
    // Uint32Array
    if let Some(arr) = value.dyn_ref::<Uint32Array>() {
        let len = arr.length();
        match len {
            1 => {
                let mut v = [0u32; 1];
                arr.copy_to(&mut v);
                return Ok(VertexValue::U32(v[0]));
            }
            2 => {
                let mut v = [0u32; 2];
                arr.copy_to(&mut v);
                return Ok(VertexValue::U32x2(v));
            }
            3 => {
                let mut v = [0u32; 3];
                arr.copy_to(&mut v);
                return Ok(VertexValue::U32x3(v));
            }
            4 => {
                let mut v = [0u32; 4];
                arr.copy_to(&mut v);
                return Ok(VertexValue::U32x4(v));
            }
            _ => {
                return Err(JsError::new(
                    "Unsupported Uint32Array length for vertex attribute",
                ));
            }
        }
    }
    // Plain JS array of numbers -> floats
    if let Some(arr) = value.dyn_ref::<js_sys::Array>() {
        let len = arr.length();
        match len {
            1 => Ok(VertexValue::F32(arr.get(0).as_f64().unwrap_or(0.0) as f32)),
            2 => Ok(VertexValue::F32x2([
                arr.get(0).as_f64().unwrap_or(0.0) as f32,
                arr.get(1).as_f64().unwrap_or(0.0) as f32,
            ])),
            3 => Ok(VertexValue::F32x3([
                arr.get(0).as_f64().unwrap_or(0.0) as f32,
                arr.get(1).as_f64().unwrap_or(0.0) as f32,
                arr.get(2).as_f64().unwrap_or(0.0) as f32,
            ])),
            4 => Ok(VertexValue::F32x4([
                arr.get(0).as_f64().unwrap_or(0.0) as f32,
                arr.get(1).as_f64().unwrap_or(0.0) as f32,
                arr.get(2).as_f64().unwrap_or(0.0) as f32,
                arr.get(3).as_f64().unwrap_or(0.0) as f32,
            ])),
            _ => Err(JsError::new(
                "Unsupported array length for vertex attribute",
            )),
        }
    } else if let Some(n) = value.as_f64() {
        Ok(VertexValue::F32(n as f32))
    } else {
        Err(JsError::new("Cannot convert value to a vertex attribute"))
    }
}

fn js_to_vertex_into(value: &JsValue) -> Result<Vertex, JsError> {
    // First try direct conversion if it's already a Vertex
    if let Ok(vertex) = Vertex::try_from(value) {
        return Ok(vertex);
    }

    // Then try converting from position data (arrays/typed arrays/numbers)
    let data = js_to_f32_vec(value).ok_or_else(|| {
        JsError::new("Expected Vertex object, number, or array/typed array for position")
    })?;
    match data.len() {
        1 => Ok(Vertex::new(data[0])),
        2 => Ok(Vertex::new([data[0], data[1]])),
        3 => Ok(Vertex::new([data[0], data[1], data[2]])),
        4 => Ok(Vertex::new([data[0], data[1], data[2], data[3]])),
        _ => Err(JsError::new("Position must have 1..=4 components")),
    }
}

fn js_to_instance_into(value: &JsValue) -> Result<Instance, JsError> {
    // First try direct conversion if it's already an Instance
    if let Ok(instance) = Instance::try_from(value) {
        return Ok(instance);
    }
    // Then try converting from Vertex object
    if let Ok(vertex) = Vertex::try_from(value) {
        return Ok(vertex.create_instance());
    }
    // Finally, accept number/array/typed array -> Vertex -> Instance
    if let Ok(vertex) = js_to_vertex_into(value) {
        return Ok(vertex.create_instance());
    }
    Err(JsError::new(
        "Expected Instance, Vertex, or number/array/typed array",
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
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/vertex/new.md")]
    pub fn new_js(position: &JsValue) -> Result<Vertex, JsError> {
        js_to_vertex_into(position)
    }

    // Support Rust-style Vertex::new([...]) in JS as Vertex.new([...])
    #[wasm_bindgen(js_name = "new")]
    pub fn new_static(position: &JsValue) -> Result<Vertex, JsError> {
        js_to_vertex_into(position)
    }

    #[wasm_bindgen(js_name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set_js(&self, key: &str, value: &JsValue) -> Result<Vertex, JsError> {
        let vv = js_to_vertex_value(value)?;
        Ok(self.clone().set(key, vv))
    }

    #[wasm_bindgen(js_name = "createInstance")]
    #[lsp_doc("docs/api/geometry/vertex/create_instance.md")]
    pub fn create_instance_js(&self) -> Instance {
        self.create_instance()
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
    pub fn from_vertices_js(list: &JsValue) -> Result<Mesh, JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("Expected an array"))?;
        let m = Mesh::new();
        for v in arr.iter() {
            let vertex = js_to_vertex_into(&v)?;
            m.add_vertex(vertex);
        }
        Ok(m)
    }

    #[wasm_bindgen(js_name = "addVertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_js(&mut self, v: &JsValue) -> Result<(), JsError> {
        let vertex = js_to_vertex_into(v)?;
        self.add_vertex(vertex);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addVertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_js(&mut self, list: &JsValue) -> Result<(), JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("Expected an array"))?;
        let mut verts: Vec<Vertex> = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            let vertex = js_to_vertex_into(&v)?;
            verts.push(vertex);
        }
        self.add_vertices(verts);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addInstance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_js(&mut self, item: &JsValue) -> Result<(), JsError> {
        let inst = js_to_instance_into(item)?;
        self.add_instance(inst);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addInstances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_js(&mut self, list: &JsValue) -> Result<(), JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("Expected an array"))?;
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
    pub fn set_instance_count_js(&mut self, n: u32) {
        self.set_instance_count(n);
    }

    #[wasm_bindgen(js_name = "clearInstanceCount")]
    #[lsp_doc("docs/api/geometry/mesh/clear_instance_count.md")]
    pub fn clear_instance_count_js(&mut self) {
        self.clear_instance_count();
    }
}

// -----------------------------
// Quad (WASM bindings)
// -----------------------------
fn js_to_f32x2(value: &JsValue) -> Result<[f32; 2], JsError> {
    let v =
        js_to_f32_vec(value).ok_or_else(|| JsError::new("Expected [x, y] array or typed array"))?;
    if v.len() != 2 {
        return Err(JsError::new("Expected [x, y] with exactly 2 numbers"));
    }
    Ok([v[0], v[1]])
}

#[wasm_bindgen]
impl Quad {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/geometry/quad/new.md")]
    pub fn new_js(min: &JsValue, max: &JsValue) -> Result<Quad, JsError> {
        let min2 = js_to_f32x2(min)?;
        let max2 = js_to_f32x2(max)?;
        Ok(Quad::new(min2, max2))
    }

    #[wasm_bindgen(js_name = "getMesh")]
    #[lsp_doc("docs/api/geometry/quad/get_mesh.md")]
    pub fn get_mesh_js(&self) -> Mesh {
        self.get_mesh()
    }
}
