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
            "Unsupported value for Vertex.set (expected number or sequence)",
        ))
    }
}

#[pymethods]
impl Vertex {
    #[new]
    #[lsp_doc("docs/api/core/vertex/new.md")]
    pub fn new_py(position: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let any = position.bind(py);
            if let Ok(v) = any.extract::<f32>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<(f32, f32)>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<(f32, f32, f32)>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<(f32, f32, f32, f32)>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<(u32, u32)>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<(u32, u32, u32)>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<[f32; 2]>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<[f32; 3]>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<[f32; 4]>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<[u32; 2]>() {
                return Ok(Vertex::new(v));
            }
            if let Ok(v) = any.extract::<[u32; 3]>() {
                return Ok(Vertex::new(v));
            }
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported position (expected number or sequence of 2..4 floats, or 2..3 u32)",
            ))
        })
    }

    #[pyo3(name = "set")]
    #[lsp_doc("docs/api/core/vertex/set.md")]
    pub fn set_py(&self, key: &str, value: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| -> PyResult<Self> {
            let vv = py_to_vertex_value(value.bind(py))?;
            Ok(self.clone().set(key, vv))
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
                // Try direct extraction first (already a Vertex)
                if let Ok(v) = item.extract::<Vertex>() {
                    list.push(v);
                    continue;
                }
                // Attempt number / tuple / list conversions mirroring Vertex::new_py logic
                if let Ok(vf) = item.extract::<f32>() {
                    list.push(Vertex::new(vf));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32)>() {
                    list.push(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32, f32)>() {
                    list.push(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32, f32, f32)>() {
                    list.push(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(u32, u32)>() {
                    list.push(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(u32, u32, u32)>() {
                    list.push(Vertex::new(t));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 2]>() {
                    list.push(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 3]>() {
                    list.push(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 4]>() {
                    list.push(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[u32; 2]>() {
                    list.push(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[u32; 3]>() {
                    list.push(Vertex::new(a));
                    continue;
                }
                return Err(PyErr::new::<PyTypeError, _>(
                    "Unsupported vertex item (expected Vertex or number/sequence)",
                ));
            }
            Ok(Mesh::from_vertices(list))
        })
    }

    #[pyo3(name = "add_vertex")]
    #[lsp_doc("docs/api/core/mesh/add_vertex.md")]
    pub fn add_vertex_py(&mut self, vertex: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let any = vertex.bind(py);
            if let Ok(v) = any.extract::<Vertex>() {
                self.add_vertex(v);
                return Ok(());
            }
            if let Ok(vf) = any.extract::<f32>() {
                self.add_vertex(Vertex::new(vf));
                return Ok(());
            }
            if let Ok(t) = any.extract::<(f32, f32)>() {
                self.add_vertex(Vertex::new(t));
                return Ok(());
            }
            if let Ok(t) = any.extract::<(f32, f32, f32)>() {
                self.add_vertex(Vertex::new(t));
                return Ok(());
            }
            if let Ok(t) = any.extract::<(f32, f32, f32, f32)>() {
                self.add_vertex(Vertex::new(t));
                return Ok(());
            }
            if let Ok(t) = any.extract::<(u32, u32)>() {
                self.add_vertex(Vertex::new(t));
                return Ok(());
            }
            if let Ok(t) = any.extract::<(u32, u32, u32)>() {
                self.add_vertex(Vertex::new(t));
                return Ok(());
            }
            if let Ok(a) = any.extract::<[f32; 2]>() {
                self.add_vertex(Vertex::new(a));
                return Ok(());
            }
            if let Ok(a) = any.extract::<[f32; 3]>() {
                self.add_vertex(Vertex::new(a));
                return Ok(());
            }
            if let Ok(a) = any.extract::<[f32; 4]>() {
                self.add_vertex(Vertex::new(a));
                return Ok(());
            }
            if let Ok(a) = any.extract::<[u32; 2]>() {
                self.add_vertex(Vertex::new(a));
                return Ok(());
            }
            if let Ok(a) = any.extract::<[u32; 3]>() {
                self.add_vertex(Vertex::new(a));
                return Ok(());
            }
            Err(PyErr::new::<PyTypeError, _>(
                "Unsupported vertex (expected Vertex or number/sequence)",
            ))
        })
    }

    #[pyo3(name = "add_vertices")]
    #[lsp_doc("docs/api/core/mesh/add_vertices.md")]
    pub fn add_vertices_py(&mut self, vertices: Py<PyAny>) -> PyResult<()> {
        Python::attach(|py| -> PyResult<()> {
            let seq = vertices.bind(py).downcast::<PySequence>()?;
            let len = seq.len()?;
            for i in 0..len {
                let item = seq.get_item(i)?;
                if let Ok(v) = item.extract::<Vertex>() {
                    self.add_vertex(v);
                    continue;
                }
                if let Ok(vf) = item.extract::<f32>() {
                    self.add_vertex(Vertex::new(vf));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32)>() {
                    self.add_vertex(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32, f32)>() {
                    self.add_vertex(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(f32, f32, f32, f32)>() {
                    self.add_vertex(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(u32, u32)>() {
                    self.add_vertex(Vertex::new(t));
                    continue;
                }
                if let Ok(t) = item.extract::<(u32, u32, u32)>() {
                    self.add_vertex(Vertex::new(t));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 2]>() {
                    self.add_vertex(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 3]>() {
                    self.add_vertex(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[f32; 4]>() {
                    self.add_vertex(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[u32; 2]>() {
                    self.add_vertex(Vertex::new(a));
                    continue;
                }
                if let Ok(a) = item.extract::<[u32; 3]>() {
                    self.add_vertex(Vertex::new(a));
                    continue;
                }
                return Err(PyErr::new::<PyTypeError, _>(
                    "Unsupported vertex in list (expected Vertex or number/sequence)",
                ));
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

    #[pyo3(name = "set_instance_count")]
    #[lsp_doc("docs/api/core/mesh/set_instance_count.md")]
    pub fn set_instance_count_py(&mut self, n: u32) {
        self.set_instance_count(n);
    }

    #[pyo3(name = "clear_instance_count")]
    #[lsp_doc("docs/api/core/mesh/clear_instance_count.md")]
    pub fn clear_instance_count_py(&mut self) {
        self.clear_instance_count();
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
