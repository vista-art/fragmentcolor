pub mod app;

use pyo3::types::PyDict;

use fc::*;
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

#[derive(Clone)]
#[pyclass(name = "FragmentColor")]
struct PyFragmentColor;

#[pymethods]
impl PyFragmentColor {
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
        FragmentColor::config(options);
    }

    #[staticmethod]
    fn run() {
        FragmentColor::run();
    }
}

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
//     FragmentColor::config(options);
// }

#[pyfunction]
pub fn run() {
    FragmentColor::run();
}
