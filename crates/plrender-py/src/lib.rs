pub mod window;
use pyo3::types::PyDict;
pub use window::*;

use plr::*;
use pyo3::prelude::*;

#[derive(Clone, Default)]
#[pyclass(name = "AppOptions")]
struct PyAppOptions {
    inner: AppOptions,
}

#[derive(Clone, Default)]
#[pyclass(name = "ShapeBuilder")]
struct PyShapeBuilder {
    _inner: ShapeBuilder,
}

// generates #[pymodule] and all inner functions

#[derive(Clone)]
#[pyclass(name = "PLRender")]
struct PyPLRender;

#[pymethods]
impl PyPLRender {
    #[staticmethod]
    fn config(options: Option<&PyDict>) {
        let options = if let Some(options) = options {
            let options = options.extract::<PyAppOptions>().unwrap_or_default();
            AppOptions {
                log_level: options.inner.log_level,
                renderer: options.inner.renderer,
            }
        } else {
            AppOptions::default()
        };
        PLRender::config(options);
    }

    #[staticmethod]
    fn run() {
        PLRender::run();
    }
}

#[derive(Clone)]
#[pyclass(name = "PLRender")]
struct PyTarget;

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
