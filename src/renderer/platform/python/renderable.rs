use crate::{Frame, Pass, PassObject, PyPassIterator, Renderable, Shader};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySequence};
use std::sync::Arc;

#[pyclass]
pub struct PyRenderable {
    iterator: PyPassIterator,
}

impl<'py> From<&'py Bound<'py, Frame>> for PyRenderable {
    fn from(frame: &Bound<'py, Frame>) -> Self {
        Python::attach(|_| -> PyResult<Self> {
            let iterator = frame.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap()
    }
}

impl<'py> From<&'py Bound<'py, Shader>> for PyRenderable {
    fn from(shader: &Bound<'py, Shader>) -> Self {
        Python::attach(|_| -> PyResult<Self> {
            let iterator = shader.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap()
    }
}

impl<'py> From<&'py Bound<'py, Pass>> for PyRenderable {
    fn from(pass: &Bound<'py, Pass>) -> Self {
        Python::attach(|_| -> PyResult<Self> {
            let iterator = pass.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap()
    }
}

impl PyRenderable {
    /// Build a renderable from a Python object:
    /// - Frame, Pass, Shader
    /// - Or a Python sequence (list/tuple) of these; collects all passes across items
    pub fn from_any<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        // Try object with renderable_type()
        if let Ok(rtype) = obj
            .call_method0("renderable_type")
            .and_then(|v| v.extract::<String>())
        {
            return match rtype.as_str() {
                "Frame" => {
                    let frame = obj.downcast::<Frame>()?;
                    Ok(PyRenderable::from(frame))
                }
                "Pass" => {
                    let pass = obj.downcast::<Pass>()?;
                    Ok(PyRenderable::from(pass))
                }
                "Shader" => {
                    let shader = obj.downcast::<Shader>()?;
                    Ok(PyRenderable::from(shader))
                }
                _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Expected Frame, Pass, or Shader",
                )),
            };
        }

        // Try a Python sequence (list/tuple) of renderables
        if let Ok(seq) = obj.downcast::<PySequence>() {
            let len = seq.len()?;
            let mut all: Vec<Arc<PassObject>> = Vec::with_capacity(len as usize);
            for i in 0..len {
                let item = seq.get_item(i)?;
                // Each item must expose passes()
                let iter = item
                    .call_method0("passes")?
                    .extract::<PyPassIterator>()?;
                for p in iter.0.into_iter() {
                    all.push(p);
                }
            }
            return Ok(Self {
                iterator: PyPassIterator(all),
            });
        }

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a Frame, Pass, Shader, or a sequence of them",
        ))
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iterator.0.iter().map(|pass| pass.as_ref())
    }
}
