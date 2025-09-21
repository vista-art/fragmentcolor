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
    if let Ok(seq) = obj.downcast::<PySequence>() {
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
            _ => unreachable!(),
        }
    } else {
        Err(PyErr::new::<PyTypeError, _>(
            "Unsupported value for Vertex.with (expected number or sequence)",
        ))
    }
}

fn py_to_vec4_and_dims(obj: &Bound<'_, PyAny>) -> PyResult<(glam::Vec4, u8)> {
    // scalar float
    if let Ok(f) = obj.extract::<f32>() {
        return Ok((glam::Vec4::new(f, 0.0, 0.0, 1.0), 1));
    }
    // handle tuples/lists
    if let Ok(seq) = obj.downcast::<PySequence>() {
        let len = seq.len()?;
        if !(1..=4).contains(&len) {
            return Err(PyErr::new::<PyTypeError, _>(format!(
                "Position must have 1..4 components, got {}",
                len
            )));
        }
        let mut v = [0.0f32; 4];
        for i in 0..len {
            let item = seq.get_item(i)?;
            if let Ok(f) = item.extract::<f32>() {
                v[i] = f;
            } else if let Ok(ii) = item.extract::<i64>() {
                v[i] = ii as f32;
            } else {
                return Err(PyErr::new::<PyTypeError, _>(
                    "Position elements must be numeric",
                ));
            }
        }
        return Ok((
            glam::Vec4::new(v[0], v[1], v[2], if len == 4 { v[3] } else { 1.0 }),
            len as u8,
        ));
    }

    Err(PyErr::new::<PyTypeError, _>(
        "Unsupported position (expected number or sequence)",
    ))
}

#[pymethods]
impl Vertex {
    #[new]
    #[lsp_doc("docs/api/core/vertex/new.md")]
    pub fn new_py(position: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let (v4, dims) = py_to_vec4_and_dims(&position.bind(py))?;
            Ok(Vertex {
                position: crate::mesh::builtins::VertexPosition(v4),
                dimensions: dims,
                properties: std::collections::HashMap::new(),
                prop_locations: std::collections::HashMap::new(),
                next_location: 1,
            })
        })
    }

    #[pyo3(name = "with")]
    #[lsp_doc("docs/api/core/vertex/with.md")]
    pub fn with_py(&self, key: &str, value: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let vv = py_to_vertex_value(&value.bind(py))?;
            Ok(self.clone().with(key, vv))
        })
    }

    #[pyo3(name = "create_instance")]
    #[lsp_doc("docs/api/core/vertex/create_instance.md")]
    pub fn create_instance_py(&self) -> Instance {
        self.create_instance()
    }
}

#[pymethods]
impl Instance {
    #[new]
    pub fn new_py() -> Self {
        Self::default()
    }
}

#[pymethods]
impl Mesh {
    #[new]
    #[lsp_doc("docs/api/core/mesh/new.md")]
    pub fn new_py() -> Self {
        Self::new()
    }

    #[staticmethod]
    #[pyo3(name = "from_vertices")]
    #[lsp_doc("docs/api/core/mesh/from_vertices.md")]
    pub fn from_vertices_py(vertices: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let seq = vertices.bind(py).downcast::<PySequence>()?;
            let len = seq.len()?;
            let mut list: Vec<Vertex> = Vec::with_capacity(len);
            for i in 0..len {
                let item = seq.get_item(i)?;
                let v = item.extract::<Vertex>()?;
                list.push(v);
            }
            Ok(Mesh::from_vertices(list))
        })
    }

    #[pyo3(name = "add_vertex")]
    #[lsp_doc("docs/api/core/mesh/add_vertex.md")]
    pub fn add_vertex_py(&mut self, v: Vertex) {
        self.add_vertex(v)
    }

    #[pyo3(name = "add_vertices")]
    #[lsp_doc("docs/api/core/mesh/add_vertices.md")]
    pub fn add_vertices_py(&mut self, vertices: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = vertices.bind(py).downcast::<PySequence>()?;
            let len = seq.len()?;
            for i in 0..len {
                let item = seq.get_item(i)?;
                let v = item.extract::<Vertex>()?;
                self.add_vertex(v);
            }
            Ok(())
        })
    }

    #[pyo3(name = "add_instance")]
    #[lsp_doc("docs/api/core/mesh/add_instance.md")]
    pub fn add_instance_py(&mut self, item: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let any = item.bind(py);
            if let Ok(v) = any.extract::<Vertex>() {
                self.add_instance(v);
                return Ok(());
            }
            if let Ok(i) = any.extract::<Instance>() {
                self.add_instance(i);
                return Ok(());
            }
            Err(PyErr::new::<PyTypeError, _>(
                "Expected a Vertex or Instance",
            ))
        })
    }

    #[pyo3(name = "add_instances")]
    #[lsp_doc("docs/api/core/mesh/add_instances.md")]
    pub fn add_instances_py(&mut self, items: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = items.bind(py).downcast::<PySequence>()?;
            let len = seq.len()?;
            for i in 0..len {
                let it = seq.get_item(i)?;
                if let Ok(v) = it.extract::<Vertex>() {
                    self.add_instance(v);
                } else if let Ok(inst) = it.extract::<Instance>() {
                    self.add_instance(inst);
                } else {
                    return Err(PyErr::new::<PyTypeError, _>(
                        "Expected a list of Vertex or Instance",
                    ));
                }
            }
            Ok(())
        })
    }

    #[pyo3(name = "clear_instances")]
    #[lsp_doc("docs/api/core/mesh/clear_instances.md")]
    pub fn clear_instances_py(&mut self) {
        self.clear_instances();
    }
}

/// Tiny factory class to construct typed VertexValue variants from Python.
#[pyclass(name = "VertexValue")]
pub struct PyVertexValue;

#[pymethods]
impl PyVertexValue {
    #[staticmethod]
    pub fn f32(x: f32) -> VertexValue {
        VertexValue::F32(x)
    }
    #[staticmethod]
    pub fn f32x2(x: [f32; 2]) -> VertexValue {
        VertexValue::F32x2(x)
    }
    #[staticmethod]
    pub fn f32x3(x: [f32; 3]) -> VertexValue {
        VertexValue::F32x3(x)
    }
    #[staticmethod]
    pub fn f32x4(x: [f32; 4]) -> VertexValue {
        VertexValue::F32x4(x)
    }
    #[staticmethod]
    pub fn u32(x: u32) -> VertexValue {
        VertexValue::U32(x)
    }
    #[staticmethod]
    pub fn u32x2(x: [u32; 2]) -> VertexValue {
        VertexValue::U32x2(x)
    }
    #[staticmethod]
    pub fn u32x3(x: [u32; 3]) -> VertexValue {
        VertexValue::U32x3(x)
    }
    #[staticmethod]
    pub fn u32x4(x: [u32; 4]) -> VertexValue {
        VertexValue::U32x4(x)
    }
    #[staticmethod]
    pub fn i32(x: i32) -> VertexValue {
        VertexValue::I32(x)
    }
    #[staticmethod]
    pub fn i32x2(x: [i32; 2]) -> VertexValue {
        VertexValue::I32x2(x)
    }
    #[staticmethod]
    pub fn i32x3(x: [i32; 3]) -> VertexValue {
        VertexValue::I32x3(x)
    }
    #[staticmethod]
    pub fn i32x4(x: [i32; 4]) -> VertexValue {
        VertexValue::I32x4(x)
    }
}
