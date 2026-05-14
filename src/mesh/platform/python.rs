#![cfg(python)]

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySequence};

use crate::mesh::{Instance, Mesh, Vertex, VertexValue};
use lsp_doc::lsp_doc;

fn py_to_vertex_value(obj: &Bound<'_, PyAny>) -> PyResult<VertexValue> {
    // First, try direct extraction of VertexValue (from our factory return)
    if let Ok(vv) = obj.extract::<VertexValue>() {
        return Ok(vv);
    }

    // Number => F32
    if let Ok(f) = obj.extract::<f32>() {
        return Ok(VertexValue::F32(f));
    }
    if let Ok(i) = obj.extract::<i64>() {
        if i >= 0 && i <= u32::MAX as i64 {
            return Ok(VertexValue::U32(i as u32));
        } else {
            return Ok(VertexValue::I32(i as i32));
        }
    }

    // Sequence => choose F32xN (fallback), or integer variants if all ints
    if let Ok(seq) = obj.cast::<PySequence>() {
        let len = seq.len()?;
        if !(1..=4).contains(&len) {
            return Err(PyErr::new::<PyTypeError, _>(format!(
                "Expected a sequence of length 1..4, got {}",
                len
            )));
        }
        // Collect as f64 to detect ints vs floats
        let mut floats: Vec<f64> = Vec::with_capacity(len);
        let mut all_int = true;
        for i in 0..len {
            let item = seq.get_item(i)?;
            if let Ok(v) = item.extract::<i64>() {
                floats.push(v as f64);
            } else if let Ok(vf) = item.extract::<f64>() {
                all_int = false;
                floats.push(vf);
            } else {
                return Err(PyErr::new::<PyTypeError, _>(
                    "Sequence elements must be numbers",
                ));
            }
        }
        // Map to variants
        match len {
            1 => {
                if all_int {
                    let v = floats[0] as i64;
                    if v >= 0 && v <= u32::MAX as i64 {
                        Ok(VertexValue::U32(v as u32))
                    } else {
                        Ok(VertexValue::I32(v as i32))
                    }
                } else {
                    Ok(VertexValue::F32(floats[0] as f32))
                }
            }
            2 => {
                if all_int {
                    let a = [floats[0] as i64, floats[1] as i64];
                    if a.iter().all(|v| *v >= 0 && *v <= u32::MAX as i64) {
                        Ok(VertexValue::U32x2([a[0] as u32, a[1] as u32]))
                    } else {
                        Ok(VertexValue::I32x2([a[0] as i32, a[1] as i32]))
                    }
                } else {
                    Ok(VertexValue::F32x2([floats[0] as f32, floats[1] as f32]))
                }
            }
            3 => {
                if all_int {
                    let a = [floats[0] as i64, floats[1] as i64, floats[2] as i64];
                    if a.iter().all(|v| *v >= 0 && *v <= u32::MAX as i64) {
                        Ok(VertexValue::U32x3([a[0] as u32, a[1] as u32, a[2] as u32]))
                    } else {
                        Ok(VertexValue::I32x3([a[0] as i32, a[1] as i32, a[2] as i32]))
                    }
                } else {
                    Ok(VertexValue::F32x3([
                        floats[0] as f32,
                        floats[1] as f32,
                        floats[2] as f32,
                    ]))
                }
            }
            4 => {
                if all_int {
                    let a = [
                        floats[0] as i64,
                        floats[1] as i64,
                        floats[2] as i64,
                        floats[3] as i64,
                    ];
                    if a.iter().all(|v| *v >= 0 && *v <= u32::MAX as i64) {
                        Ok(VertexValue::U32x4([
                            a[0] as u32,
                            a[1] as u32,
                            a[2] as u32,
                            a[3] as u32,
                        ]))
                    } else {
                        Ok(VertexValue::I32x4([
                            a[0] as i32,
                            a[1] as i32,
                            a[2] as i32,
                            a[3] as i32,
                        ]))
                    }
                } else {
                    Ok(VertexValue::F32x4([
                        floats[0] as f32,
                        floats[1] as f32,
                        floats[2] as f32,
                        floats[3] as f32,
                    ]))
                }
            }
            _ => Err(PyErr::new::<PyTypeError, _>(
                "(unreachable): Unexpected sequence length",
            )),
        }
    } else {
        Err(PyErr::new::<PyTypeError, _>(
            "Unsupported value for Vertex.set (expected number or sequence)",
        ))
    }
}

fn py_to_vertex(obj: &Bound<'_, PyAny>) -> PyResult<Vertex> {
    // Try direct extraction first (already a Vertex)
    if let Ok(v) = obj.extract::<Vertex>() {
        return Ok(v);
    }

    // Try extracting various number/tuple/array types
    if let Ok(v) = obj.extract::<f32>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<(f32, f32)>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<(f32, f32, f32)>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<(f32, f32, f32, f32)>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<(u32, u32)>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<(u32, u32, u32)>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<[f32; 2]>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<[f32; 3]>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<[f32; 4]>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<[u32; 2]>() {
        return Ok(Vertex::new(v));
    }
    if let Ok(v) = obj.extract::<[u32; 3]>() {
        return Ok(Vertex::new(v));
    }

    Err(PyErr::new::<PyTypeError, _>(
        "Unsupported vertex type (expected Vertex or number/sequence)",
    ))
}

fn py_to_instance(obj: &Bound<'_, PyAny>) -> PyResult<Instance> {
    use pyo3::types::PyDict;

    // 1. Already an Instance.
    if let Ok(i) = obj.extract::<Instance>() {
        return Ok(i);
    }

    // 2. Plain dict of named attributes: {"key": value, ...} → Instance::new().set(...)
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut instance = Instance::new();
        for (key, value) in dict.iter() {
            let k: String = key.extract()?;
            let vv = py_to_vertex_value(&value)?;
            instance = instance.set(&k, vv);
        }
        return Ok(instance);
    }

    // 3. Vertex (backward-compat: convert via create_instance, dropping position).
    if let Ok(vertex) = py_to_vertex(obj) {
        return Ok(vertex.create_instance());
    }

    Err(PyErr::new::<PyTypeError, _>(
        "Expected an Instance, a dict of named attributes, or a Vertex",
    ))
}

#[pymethods]
impl Vertex {
    #[new]
    #[lsp_doc("docs/api/geometry/vertex/new.md")]
    pub fn new_py(position: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| py_to_vertex(position.bind(py)))
    }

    #[staticmethod]
    #[pyo3(name = "pbr")]
    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    pub fn pbr_py(position: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            // Build the base Vertex from the position, then layer the
            // PBR identity defaults via the same `.set(...)` chain
            // `Vertex::pbr` uses on the Rust side.
            let v = py_to_vertex(position.bind(py))?;
            Ok(v.set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
                .set(Self::UV0, [0.0_f32, 0.0])
                .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
                .set(Self::UV1, [0.0_f32, 0.0])
                .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0]))
        })
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let vv = py_to_vertex_value(value.bind(py))?;
            Ok(self.clone().set(key, vv))
        })
    }

    #[pyo3(name = "create_instance")]
    #[lsp_doc("docs/api/geometry/vertex/create_instance.md")]
    pub fn create_instance_py(&self) -> Instance {
        self.create_instance()
    }
}

#[pymethods]
impl Instance {
    // Instance is a helper value type; its constructors/methods are not
    // separately documented as user API. Mark as doc(hidden) to satisfy
    // the parity audit's unlabeled-export check.
    #[doc(hidden)]
    #[new]
    pub fn new_py() -> Self {
        Self::default()
    }

    #[doc(hidden)]
    #[pyo3(name = "set")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let vv = py_to_vertex_value(value.bind(py))?;
            Ok(self.clone().set(key, vv))
        })
    }
}

#[pymethods]
impl Mesh {
    #[new]
    #[lsp_doc("docs/api/geometry/mesh/new.md")]
    pub fn new_py() -> Self {
        Self::new()
    }

    #[staticmethod]
    #[pyo3(name = "from_vertices")]
    #[lsp_doc("docs/api/geometry/mesh/from_vertices.md")]
    pub fn from_vertices_py(vertices: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let seq = vertices.bind(py).cast::<PySequence>()?;
            let len = seq.len()?;
            let mut list: Vec<Vertex> = Vec::with_capacity(len);
            for i in 0..len {
                let item = seq.get_item(i)?;
                list.push(py_to_vertex(&item)?);
            }
            Ok(Mesh::from_vertices(list))
        })
    }

    #[pyo3(name = "add_vertex")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex_py(&mut self, vertex: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let vertex = py_to_vertex(vertex.bind(py))?;
            self.add_vertex(vertex);
            Ok(())
        })
    }

    #[pyo3(name = "add_vertices")]
    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices_py(&mut self, vertices: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = vertices.bind(py).cast::<PySequence>()?;
            let len = seq.len()?;
            for i in 0..len {
                let item = seq.get_item(i)?;
                let vertex = py_to_vertex(&item)?;
                self.add_vertex(vertex);
            }
            Ok(())
        })
    }

    #[pyo3(name = "add_instance")]
    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance_py(&mut self, item: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let instance = py_to_instance(item.bind(py))?;
            self.add_instance(instance);
            Ok(())
        })
    }

    #[pyo3(name = "add_instances")]
    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances_py(&mut self, items: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = items.bind(py).cast::<PySequence>()?;
            let len = seq.len()?;
            for i in 0..len {
                let item = seq.get_item(i)?;
                let instance = py_to_instance(&item)?;
                self.add_instance(instance);
            }
            Ok(())
        })
    }

    #[pyo3(name = "clear_instances")]
    #[lsp_doc("docs/api/geometry/mesh/clear_instances.md")]
    pub fn clear_instances_py(&mut self) {
        self.clear_instances();
    }

    #[pyo3(name = "set_instance_count")]
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count_py(&mut self, n: u32) {
        self.set_instance_count(n);
    }

    #[pyo3(name = "set_indices")]
    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices_py(&mut self, indices: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = indices.bind(py).cast::<PySequence>()?;
            let len = seq.len()?;
            let mut out: Vec<u32> = Vec::with_capacity(len);
            for i in 0..len {
                let item = seq.get_item(i)?;
                out.push(item.extract::<u32>()?);
            }
            self.set_indices(out);
            Ok(())
        })
    }

    #[pyo3(name = "clear_indices")]
    #[lsp_doc("docs/api/geometry/mesh/clear_indices.md")]
    pub fn clear_indices_py(&mut self) {
        self.clear_indices();
    }
}

/// Tiny factory class to construct typed VertexValue variants from Python.
/// Internal helper — not part of the public documented API.
#[pyclass(name = "VertexValue")]
pub struct PyVertexValue;

#[pymethods]
impl PyVertexValue {
    #[doc(hidden)]
    #[staticmethod]
    pub fn f32(x: f32) -> VertexValue {
        VertexValue::F32(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn f32x2(x: [f32; 2]) -> VertexValue {
        VertexValue::F32x2(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn f32x3(x: [f32; 3]) -> VertexValue {
        VertexValue::F32x3(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn f32x4(x: [f32; 4]) -> VertexValue {
        VertexValue::F32x4(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn u32(x: u32) -> VertexValue {
        VertexValue::U32(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn u32x2(x: [u32; 2]) -> VertexValue {
        VertexValue::U32x2(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn u32x3(x: [u32; 3]) -> VertexValue {
        VertexValue::U32x3(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn u32x4(x: [u32; 4]) -> VertexValue {
        VertexValue::U32x4(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn i32(x: i32) -> VertexValue {
        VertexValue::I32(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn i32x2(x: [i32; 2]) -> VertexValue {
        VertexValue::I32x2(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn i32x3(x: [i32; 3]) -> VertexValue {
        VertexValue::I32x3(x)
    }
    #[doc(hidden)]
    #[staticmethod]
    pub fn i32x4(x: [i32; 4]) -> VertexValue {
        VertexValue::I32x4(x)
    }
}
