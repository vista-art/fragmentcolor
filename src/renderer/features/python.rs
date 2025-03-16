use std::sync::Arc;

use crate::{
    Frame, InitializationError, Pass, PassObject, RenderCanvasTarget, Renderable, Renderer, Shader,
};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyPassIterator(pub Vec<Arc<PassObject>>);

impl PyPassIterator {
    pub fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.0.iter().map(|pass| pass.as_ref())
    }
}

impl IntoIterator for PyPassIterator {
    type Item = Arc<PassObject>;
    type IntoIter = std::vec::IntoIter<Arc<PassObject>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[pyclass]
pub struct PyRenderable {
    iterator: PyPassIterator,
}

impl<'py> From<&'py Bound<'py, Frame>> for PyRenderable {
    fn from(frame: &Bound<'py, Frame>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = frame.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl<'py> From<&'py Bound<'py, Shader>> for PyRenderable {
    fn from(shader: &Bound<'py, Shader>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = shader.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl<'py> From<&'py Bound<'py, Pass>> for PyRenderable {
    fn from(pass: &Bound<'py, Pass>) -> Self {
        let iterator = Python::with_gil(|_| -> PyResult<Self> {
            let iterator = pass.call_method0("passes")?.extract::<PyPassIterator>()?;

            Ok(Self { iterator })
        })
        .unwrap();

        iterator
    }
}

impl Renderable for PyRenderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iterator.0.iter().map(|pass| pass.as_ref())
    }
}

#[pymethods]
impl Renderer {
    #[new]
    /// Creates a headless renderer by default
    pub fn new_py() -> Result<Renderer, InitializationError> {
        pollster::block_on(Self::headless())
    }

    #[pyo3(name = "headless")]
    #[staticmethod]
    /// Creates a headless renderer
    pub async fn headless_py() -> Result<Renderer, InitializationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = crate::platform::all::request_headless_adapter(&instance).await?;
        let (device, queue) = crate::platform::all::request_device(&adapter).await?;

        Ok(Renderer::init(device, queue))
    }

    #[pyo3(name = "render")]
    /// Renders a Frame or Shader to a Target.
    pub fn render_py(
        &self,
        renderable: PyObject,
        target: &RenderCanvasTarget,
    ) -> Result<(), PyErr> {
        Python::with_gil(|py| -> Result<(), PyErr> {
            let renderable_type = renderable
                .call_method0(py, "renderable_type")?
                .extract::<String>(py)?;

            match renderable_type.as_str() {
                "Frame" => {
                    let frame = renderable.bind(py).downcast::<Frame>()?;
                    let renderable = PyRenderable::from(frame);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                "Pass" => {
                    let pass = renderable.bind(py).downcast::<Pass>()?;
                    let renderable = PyRenderable::from(pass);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                "Shader" => {
                    let shader = renderable.bind(py).downcast::<Shader>()?;
                    let renderable = PyRenderable::from(shader);

                    self.render(&renderable, target)?;

                    Ok(())
                }
                _ => Err(PyErr::new::<PyTypeError, _>(
                    "Expected a Frame, Pass or Shader object",
                )),
            }
        })?;

        Ok(())
    }
}
