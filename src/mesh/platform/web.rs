#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::mesh::{Instance, Mesh, Quad, Vertex, VertexValue};
use crate::platforms::web::{read_numbers, read_rows, read_unsigned};

// -----------------------------
// JS helpers
// -----------------------------
fn js_to_f32_vec(value: &JsValue) -> Option<Vec<f32>> {
    if let Some(number) = value.as_f64() {
        return Some(vec![number as f32]);
    }
    let numbers = read_numbers(value)?;
    Some(numbers.into_iter().map(|number| number as f32).collect())
}

/// Reads a vertex attribute value. A `VertexValue` keeps its exact type, an
/// `Int32Array` or `Uint32Array` gives integers, and any other number or list
/// of 1 to 4 numbers gives floats. A non-numeric item throws.
fn js_to_vertex_value(value: &JsValue) -> Result<VertexValue, JsError> {
    if let Ok(value) = JsVertexValue::try_from(value) {
        return Ok(value.0);
    }
    if let Some(array) = value.dyn_ref::<js_sys::Int32Array>() {
        return read_ints(&array.to_vec());
    }
    if let Some(array) = value.dyn_ref::<js_sys::Uint32Array>() {
        return read_uints(&array.to_vec());
    }
    let numbers = js_to_f32_vec(value).ok_or_else(|| {
        JsError::new(
            "a vertex attribute must be a number, 1 to 4 numbers, a typed array, or a VertexValue",
        )
    })?;
    read_floats(&numbers)
}

fn read_floats(values: &[f32]) -> Result<VertexValue, JsError> {
    Ok(match *values {
        [x] => VertexValue::F32(x),
        [x, y] => VertexValue::F32x2([x, y]),
        [x, y, z] => VertexValue::F32x3([x, y, z]),
        [x, y, z, w] => VertexValue::F32x4([x, y, z, w]),
        _ => return Err(reject_length(values.len())),
    })
}

fn read_ints(values: &[i32]) -> Result<VertexValue, JsError> {
    Ok(match *values {
        [x] => VertexValue::I32(x),
        [x, y] => VertexValue::I32x2([x, y]),
        [x, y, z] => VertexValue::I32x3([x, y, z]),
        [x, y, z, w] => VertexValue::I32x4([x, y, z, w]),
        _ => return Err(reject_length(values.len())),
    })
}

fn read_uints(values: &[u32]) -> Result<VertexValue, JsError> {
    Ok(match *values {
        [x] => VertexValue::U32(x),
        [x, y] => VertexValue::U32x2([x, y]),
        [x, y, z] => VertexValue::U32x3([x, y, z]),
        [x, y, z, w] => VertexValue::U32x4([x, y, z, w]),
        _ => return Err(reject_length(values.len())),
    })
}

fn reject_length(length: usize) -> JsError {
    JsError::new(&format!(
        "a vertex attribute has 1 to 4 components, got {length}"
    ))
}

/// Reads a vertex position: a `Vertex`, a number, or 1 to 4 numbers.
fn js_to_vertex_into(value: &JsValue) -> Result<Vertex, JsError> {
    if let Ok(vertex) = Vertex::try_from(value) {
        return Ok(vertex);
    }
    let data = js_to_f32_vec(value).ok_or_else(|| {
        JsError::new("Expected Vertex object, number, or array/typed array for position")
    })?;
    match *data {
        [x] => Ok(Vertex::new(x)),
        [x, y] => Ok(Vertex::new([x, y])),
        [x, y, z] => Ok(Vertex::new([x, y, z])),
        [x, y, z, w] => Ok(Vertex::new([x, y, z, w])),
        _ => Err(JsError::new("Position must have 1..=4 components")),
    }
}

/// Reads a vertex for the `Mesh` methods: anything `new Vertex` takes, or the
/// `[position, uv]` and `[position, uv, color]` forms of Rust's `Into<Vertex>`.
fn read_vertex(value: &JsValue) -> Result<Vertex, JsError> {
    let Some(parts) = read_rows(value).filter(|parts| parts.len() > 1) else {
        return js_to_vertex_into(value);
    };
    let floats = |part: &Vec<f64>| part.iter().map(|&n| n as f32).collect::<Vec<f32>>();
    let parts: Vec<Vec<f32>> = parts.iter().map(floats).collect();
    match parts.as_slice() {
        [position, uv] if position.len() == 3 && uv.len() == 2 => {
            let (position, uv) = ([position[0], position[1], position[2]], [uv[0], uv[1]]);
            Ok(Vertex::from((position, uv)))
        }
        [position, uv, color] if position.len() == 3 && uv.len() == 2 && color.len() == 4 => {
            let position = [position[0], position[1], position[2]];
            let color = [color[0], color[1], color[2], color[3]];
            Ok(Vertex::from((position, [uv[0], uv[1]], color)))
        }
        _ => Err(JsError::new(
            "a vertex given in parts is [position, uv] or [position, uv, color], with 3, 2 and 4 numbers",
        )),
    }
}

fn js_to_instance_into(value: &JsValue) -> Result<Instance, JsError> {
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
                .ok_or_else(|| JsError::new("Instance attribute keys must be strings"))?;
            let vv = js_to_vertex_value(&pair.get(1))?;
            instance = instance.set(&key, vv);
        }
        return Ok(instance);
    }
    Err(JsError::new(
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
    pub fn new_js(position: &JsValue) -> Result<Vertex, JsError> {
        js_to_vertex_into(position)
    }

    // Support Rust-style Vertex::new([...]) in JS as Vertex.new([...])
    // JS compat shim — same entry point as the constructor; not separately documented.
    #[doc(hidden)]
    #[wasm_bindgen(js_name = "new")]
    pub fn new_static(position: &JsValue) -> Result<Vertex, JsError> {
        js_to_vertex_into(position)
    }

    #[wasm_bindgen(js_name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_js(position: &JsValue) -> Result<Vertex, JsError> {
        let v = js_to_vertex_into(position)?;
        Ok(v.set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0]))
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
    pub fn set_js(&self, key: &str, value: &JsValue) -> Result<Instance, JsError> {
        let vv = js_to_vertex_value(value)?;
        Ok(self.clone().set(key, vv))
    }
}

// -----------------------------
// VertexValue (WASM bindings)
// -----------------------------

/// A vertex attribute value of an exact type, such as `VertexValue.u32(3)` for
/// a `u32` shader input. JS reads plain numbers and arrays as floats. The
/// factories match Python's, and an integer that doesn't fit its type throws.
#[wasm_bindgen(js_name = "VertexValue")]
#[derive(Clone)]
pub struct JsVertexValue(VertexValue);

impl crate::FcKind for JsVertexValue {
    const KIND: &'static str = "VertexValue";
}

crate::impl_js_bridge!(JsVertexValue, crate::mesh::error::MeshError, "VertexValue");

#[wasm_bindgen(js_class = "VertexValue")]
impl JsVertexValue {
    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "__fc_kind")]
    pub fn js_fc_kind(&self) -> String {
        <Self as crate::FcKind>::KIND.into()
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "f32")]
    pub fn f32_js(x: f32) -> Self {
        Self(VertexValue::F32(x))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "f32x2")]
    pub fn f32x2_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        let [a, b] = read_components(x, "VertexValue.f32x2")?;
        Ok(Self(VertexValue::F32x2([a as f32, b as f32])))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "f32x3")]
    pub fn f32x3_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        let [a, b, c] = read_components(x, "VertexValue.f32x3")?;
        Ok(Self(VertexValue::F32x3([a as f32, b as f32, c as f32])))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "f32x4")]
    pub fn f32x4_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        let [a, b, c, d] = read_components(x, "VertexValue.f32x4")?;
        Ok(Self(VertexValue::F32x4([
            a as f32, b as f32, c as f32, d as f32,
        ])))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "u32")]
    pub fn u32_js(x: f64) -> Result<Self, JsError> {
        Ok(Self(VertexValue::U32(read_unsigned(x, "VertexValue.u32")?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "u32x2")]
    pub fn u32x2_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::U32x2(read_uint_components(
            x,
            "VertexValue.u32x2",
        )?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "u32x3")]
    pub fn u32x3_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::U32x3(read_uint_components(
            x,
            "VertexValue.u32x3",
        )?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "u32x4")]
    pub fn u32x4_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::U32x4(read_uint_components(
            x,
            "VertexValue.u32x4",
        )?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "i32")]
    pub fn i32_js(x: f64) -> Result<Self, JsError> {
        Ok(Self(VertexValue::I32(read_signed(x, "VertexValue.i32")?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "i32x2")]
    pub fn i32x2_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::I32x2(read_int_components(
            x,
            "VertexValue.i32x2",
        )?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "i32x3")]
    pub fn i32x3_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::I32x3(read_int_components(
            x,
            "VertexValue.i32x3",
        )?)))
    }

    #[doc(hidden)]
    #[wasm_bindgen(js_name = "i32x4")]
    pub fn i32x4_js(
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] x: &JsValue,
    ) -> Result<Self, JsError> {
        Ok(Self(VertexValue::I32x4(read_int_components(
            x,
            "VertexValue.i32x4",
        )?)))
    }
}

/// Reads exactly `N` numbers for a `VertexValue` factory named `name`.
fn read_components<const N: usize>(value: &JsValue, name: &str) -> Result<[f64; N], JsError> {
    let numbers = read_numbers(value).unwrap_or_default();
    numbers
        .try_into()
        .map_err(|_| JsError::new(&format!("{name}: expected an array of {N} numbers")))
}

fn read_uint_components<const N: usize>(value: &JsValue, name: &str) -> Result<[u32; N], JsError> {
    let mut out = [0; N];
    for (slot, number) in out.iter_mut().zip(read_components::<N>(value, name)?) {
        *slot = read_unsigned(number, name)?;
    }
    Ok(out)
}

fn read_int_components<const N: usize>(value: &JsValue, name: &str) -> Result<[i32; N], JsError> {
    let mut out = [0; N];
    for (slot, number) in out.iter_mut().zip(read_components::<N>(value, name)?) {
        *slot = read_signed(number, name)?;
    }
    Ok(out)
}

/// Reads a JS number as a signed 32-bit integer.
fn read_signed(value: f64, name: &str) -> Result<i32, JsError> {
    let whole = value.is_finite() && value.fract() == 0.0;
    if whole && (i32::MIN as f64..=i32::MAX as f64).contains(&value) {
        return Ok(value as i32);
    }
    Err(JsError::new(&format!(
        "{name}: expected a whole number from {} to {}, got {value}",
        i32::MIN,
        i32::MAX
    )))
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
            let vertex = read_vertex(&v)?;
            m.add_vertex(vertex);
        }
        Ok(m)
    }

    #[wasm_bindgen(js_name = "addVertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_js(&mut self, v: &JsValue) -> Result<(), JsError> {
        let vertex = read_vertex(v)?;
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
            let vertex = read_vertex(&v)?;
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
    pub fn set_instance_count_js(&mut self, count: f64) -> Result<(), JsError> {
        self.set_instance_count(read_unsigned(count, "Mesh.setInstanceCount: count")?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "setIndices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_js(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "ArrayLike<number>")] indices: &JsValue,
    ) -> Result<(), JsError> {
        if let Some(array) = indices.dyn_ref::<js_sys::Uint32Array>() {
            self.set_indices(array.to_vec());
            return Ok(());
        }
        let numbers = read_numbers(indices).ok_or_else(|| {
            JsError::new("Mesh.setIndices: expected an array or typed array of numbers")
        })?;
        let read = |(index, number): (usize, f64)| {
            read_unsigned(number, &format!("Mesh.setIndices: item {index}"))
        };
        let indices = numbers.into_iter().enumerate().map(read);
        self.set_indices(indices.collect::<Result<Vec<u32>, JsError>>()?);
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
