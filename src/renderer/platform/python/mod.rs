use crate::{FragmentColorError, Frame, Pass, Renderer, Shader, Target};
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub mod target;
pub use target::*;

#[pyclass(dict)]
#[derive(Debug)]
pub struct RenderCanvasContext {
    pub(crate) canvas: PyObject,
    _present_methods: PyObject, // @TODO figure how RenderCanvas expects me to use this

    // Reference-counted for Python.
    renderer: Option<Py<Renderer>>,
    target: Option<Py<RenderCanvasTarget>>,
}

fn clone_py<T>(object: &Py<T>) -> Py<T> {
    Python::with_gil(|py| object.clone_ref(py))
}

#[pymethods]
impl RenderCanvasContext {
    #[new]
    pub fn new(canvas: PyObject, _present_methods: PyObject) -> Self {
        RenderCanvasContext {
            canvas,
            _present_methods,
            renderer: None,
            target: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.renderer.is_some() && self.target.is_some()
    }

    // @TODO consider storing them as PyObject to be able to return them directly
    #[getter]
    pub fn renderer(&self) -> PyResult<Py<Renderer>> {
        if let Some(renderer) = &self.renderer {
            Ok(clone_py::<Renderer>(renderer))
        } else {
            Err(FragmentColorError::new_err("Renderer not initialized"))
        }
    }

    #[getter]
    pub fn target(&self) -> PyResult<Py<RenderCanvasTarget>> {
        if let Some(target) = &self.target {
            Ok(clone_py::<RenderCanvasTarget>(target))
        } else {
            Err(FragmentColorError::new_err("Target not initialized"))
        }
    }

    // duck-typed interface that a context must implement, to be usable with RenderCanvas.
    // Upstream documentation: https://rendercanvas.readthedocs.io/stable/contextapi.html
    //
    // fn canvas(&self) -> PyObject;
    // fn present(&self) -> Result<Py<PyDict>, PyErr>;
    //
    // We can't export a impl Trait block with Pyo3.

    #[getter]
    pub fn canvas(&self) -> PyObject {
        clone_py::<PyAny>(&self.canvas)
    }

    pub fn present(&self) -> Result<Py<PyDict>, PyErr> {
        Python::with_gil(|py| -> PyResult<Py<PyDict>> {
            let dict = PyDict::new(py);

            if let Some(py_target) = &self.target {
                let target = py_target.borrow(py);
                match target.get_current_frame() {
                    Ok(frame) => {
                        frame.present();
                        dict.set_item("method", "screen")?;
                    }
                    Err(e) => {
                        dict.set_item("method", "fail")?;
                        dict.set_item("message", e.to_string())?;
                    }
                }
            } else {
                dict.set_item("method", "fail")?;
                dict.set_item("message", "Target not initialized")?;
            };

            Ok(dict.unbind())
        })
    }
}

impl RenderCanvasContext {
    pub(crate) fn init_context(&mut self, renderer: Py<Renderer>, target: Py<RenderCanvasTarget>) {
        self.renderer = Some(renderer);
        self.target = Some(target);
    }
}

#[pyfunction]
/// The context hook that will be called from Python by RenderCanvas
/// When the user calls `RenderCanvas.get_context("fragmentcolor")`
pub fn rendercanvas_context_hook(
    canvas: PyObject,
    present_methods: PyObject,
) -> RenderCanvasContext {
    RenderCanvasContext::new(canvas, present_methods)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
pub fn fragmentcolor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Renderer>()?;
    m.add_class::<Shader>()?;
    m.add_class::<Pass>()?;
    m.add_class::<Frame>()?;

    // RenderCanvas API
    m.add_function(wrap_pyfunction!(rendercanvas_context_hook, m)?)?;
    m.add_class::<RenderCanvasTarget>()?;
    m.add_class::<RenderCanvasFrame>()?;
    m.add_class::<RenderCanvasContext>()?;

    // Custom error type
    m.add(
        "FragmentColorError",
        m.py().get_type::<FragmentColorError>(),
    )?;

    Ok(())
}
