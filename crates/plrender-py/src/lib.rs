pub mod window;
pub use window::*;

//use codegen::{py_module, wrap_py};
use plr::*;
use pyo3::prelude::*;
//use pyo3::types::PyDict;
use std::sync::{Arc, RwLock};

// generates #[pymodule] and all inner functions
// py_module!("plrender");

#[derive(Clone)]
#[pyclass(name = "PLRender")]
struct PyPLRender;

#[derive(Clone)]
#[pyclass(name = "PLRender")]
struct PyRenderTargetDescription;

// Implements #[pymethod] and all inner functions
// Generates implementations for PyTypeInfo, PyTypeObject, and PyClass
// wrap_py!(Scene);

// #[pyfunction]
// fn config(options: Option<&PyDict>) {
//     let options = if let Some(options) = options {
//         let options = options.extract::<PyAppOptions>().unwrap_or_default();
//         AppOptions {
//             log_level: options.inner.log_level,
//             renderer: options.inner.renderer,
//         }
//     } else {
//         AppOptions::default()
//     };
//     println!("{:?}", options);
//     PLRender::config(options);
// }

#[pyfunction]
pub fn run() {
    PLRender::run();
}

#[pyclass(name = "App")]
pub struct PyApp {
    _inner: &'static Arc<RwLock<App>>,
}

#[pymethods]
impl PyApp {
    #[new] // @TODO config options
    pub fn new() -> PyResult<Self> {
        Ok(Self {
            _inner: PLRender::app(),
        })
    }
}
