use crate::{Frame, Pass, PassObject, PyPassIterator, Renderable, Shader};
use pyo3::prelude::*;

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

impl Renderable for PyRenderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iterator.0.iter().map(|pass| pass.as_ref())
    }
}
