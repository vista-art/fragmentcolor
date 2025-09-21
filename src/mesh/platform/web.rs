#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::mesh::{Instance, Mesh, Vertex, VertexValue};

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

fn js_to_vertex_position(value: &JsValue) -> Result<Vertex, JsError> {
    let data = js_to_f32_vec(value).ok_or_else(|| {
        JsError::new("Invalid position type; expected number or array/typed array")
    })?;
    match data.len() {
        1 => Ok(Vertex::new(data[0])),
        2 => Ok(Vertex::new([data[0], data[1]])),
        3 => Ok(Vertex::new([data[0], data[1], data[2]])),
        4 => Ok(Vertex::new([data[0], data[1], data[2], data[3]])),
        _ => Err(JsError::new("Position must have 1..=4 components")),
    }
}

// -----------------------------
// JS conversions for Vertex/Instance owned via __wbg_ptr anchors
// -----------------------------
crate::impl_tryfrom_js_ref_anchor!(Vertex, crate::mesh::error::MeshError, "Vertex");

crate::impl_tryfrom_js_ref_anchor!(Instance, crate::mesh::error::MeshError, "Instance");

crate::impl_tryfrom_owned_via_ref!(Vertex, wasm_bindgen::JsValue, crate::mesh::error::MeshError);
crate::impl_tryfrom_owned_via_ref!(
    Instance,
    wasm_bindgen::JsValue,
    crate::mesh::error::MeshError
);

crate::impl_tryfrom_js_ref_anchor!(Mesh, crate::mesh::error::MeshError, "Mesh");

crate::impl_tryfrom_owned_via_ref!(Mesh, wasm_bindgen::JsValue, crate::mesh::error::MeshError);

// -----------------------------
// Vertex (WASM bindings)
// -----------------------------
#[wasm_bindgen]
impl Vertex {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/vertex/new.md")]
    pub fn new_js(position: &JsValue) -> Result<Vertex, JsError> {
        js_to_vertex_position(position)
    }

    #[wasm_bindgen(js_name = "with")]
    #[lsp_doc("docs/api/core/vertex/with.md")]
    pub fn with_js(&self, key: &str, value: &JsValue) -> Result<Vertex, JsError> {
        let vv = js_to_vertex_value(value)?;
        Ok(self.clone().with(key, vv))
    }

    #[wasm_bindgen(js_name = "createInstance")]
    #[lsp_doc("docs/api/core/vertex/create_instance.md")]
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
    #[lsp_doc("docs/api/core/mesh/new.md")]
    pub fn new_js() -> Mesh {
        Mesh::new()
    }

    #[wasm_bindgen(js_name = "fromVertices")]
    #[lsp_doc("docs/api/core/mesh/from_vertices.md")]
    pub fn from_vertices_js(list: &JsValue) -> Result<Mesh, JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("Expected an array of Vertex"))?;
        let mut m = Mesh::new();
        for v in arr.iter() {
            let vert: Vertex = (&v)
                .try_into()
                .map_err(|_| JsError::new("fromVertices: item is not a Vertex"))?;
            m.add_vertex(vert);
        }
        Ok(m)
    }

    #[wasm_bindgen(js_name = "addVertex")]
    #[lsp_doc("docs/api/core/mesh/add_vertex.md")]
    pub fn add_vertex_js(&mut self, v: &JsValue) -> Result<(), JsError> {
        let vert: Vertex = v
            .try_into()
            .map_err(|_| JsError::new("addVertex: expected a Vertex object"))?;
        self.add_vertex(vert);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addVertices")]
    #[lsp_doc("docs/api/core/mesh/add_vertices.md")]
    pub fn add_vertices_js(&mut self, list: &JsValue) -> Result<(), JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("addVertices: expected an array of Vertex"))?;
        let mut verts: Vec<Vertex> = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            let vert: Vertex = (&v)
                .try_into()
                .map_err(|_| JsError::new("addVertices: item is not a Vertex"))?;
            verts.push(vert);
        }
        self.add_vertices(verts);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addInstance")]
    #[lsp_doc("docs/api/core/mesh/add_instance.md")]
    pub fn add_instance_js(&mut self, item: &JsValue) -> Result<(), JsError> {
        // Try Instance first, then Vertex (converted to Instance)
        if let Ok(inst) = Instance::try_from(item) {
            self.add_instance(inst);
            return Ok(());
        }
        if let Ok(vtx) = Vertex::try_from(item) {
            self.add_instance(vtx);
            return Ok(());
        }
        Err(JsError::new("addInstance: expected Instance or Vertex"))
    }

    #[wasm_bindgen(js_name = "addInstances")]
    #[lsp_doc("docs/api/core/mesh/add_instances.md")]
    pub fn add_instances_js(&mut self, list: &JsValue) -> Result<(), JsError> {
        let arr = js_sys::Array::is_array(list)
            .then(|| js_sys::Array::from(list))
            .ok_or_else(|| JsError::new("addInstances: expected an array"))?;
        for item in arr.iter() {
            if let Ok(inst) = Instance::try_from(&item) {
                self.add_instance(inst);
                continue;
            }
            if let Ok(vtx) = Vertex::try_from(&item) {
                self.add_instance(vtx);
                continue;
            }
            return Err(JsError::new(
                "addInstances: items must be Instance or Vertex",
            ));
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = "clearInstances")]
    #[lsp_doc("docs/api/core/mesh/clear_instances.md")]
    pub fn clear_instances_js(&mut self) {
        self.clear_instances();
    }
}
