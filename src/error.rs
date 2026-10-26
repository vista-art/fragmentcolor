use thiserror::Error;

// Top-level catch-all error

#[derive(Error, Debug)]
pub enum FragmentColorError {
    #[error(transparent)]
    Shader(#[from] crate::shader::error::ShaderError),
    #[error(transparent)]
    Color(#[from] crate::color::error::ColorError),
    #[error(transparent)]
    Size(#[from] crate::size::error::SizeError),
    #[error(transparent)]
    Pass(#[from] crate::pass::error::PassError),
    #[error(transparent)]
    Renderer(#[from] crate::renderer::error::RendererError),
    #[error(transparent)]
    Init(#[from] crate::renderer::error::InitializationError),
    #[error(transparent)]
    Display(#[from] crate::target::error::DisplayError),
    #[cfg(not(wasm))]
    #[error("Network Request Error: {0}")]
    NetworkRequest(#[from] crate::net::NetworkError),
    #[cfg(wasm)]
    #[error("FragmentColor WASM Error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
use pyo3::{create_exception, exceptions::PyException, prelude::*};
#[cfg(python)]
create_exception!(fragment_color, PyFragmentColorError, PyException);
#[cfg(python)]
create_exception!(
    fragment_color,
    InvalidInputError,
    PyFragmentColorError,
    "A value the call can't take, such as a negative size or bytes that don't match their texture."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    UnsupportedMipmapFormatError,
    PyFragmentColorError,
    "A texture format the CPU mipmap builder can't fill."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    MalformedImageError,
    PyFragmentColorError,
    "Image bytes that can't be decoded."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    OccludedSurfaceError,
    PyFragmentColorError,
    "A window that is hidden or minimized, so render skips the frame."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    DeviceError,
    PyFragmentColorError,
    "A GPU error an error scope caught, with its kind, the label of the step that raised it and the device's own detail."
);

#[cfg(python)]
impl DeviceError {
    /// A `DeviceError` with the full `message` that carries the GPU error's `kind`,
    /// the `label` of its step and the device's own `detail`.
    pub(crate) fn raise(
        kind: crate::ErrorKind,
        label: String,
        detail: String,
        message: String,
    ) -> PyErr {
        Python::attach(|py| {
            let error = DeviceError::new_err(message);
            let set = {
                let value = error.value(py);
                value
                    .setattr("kind", kind)
                    .and_then(|()| value.setattr("label", label))
                    .and_then(|()| value.setattr("detail", detail))
            };
            match set {
                Ok(()) => error,
                Err(failure) => failure,
            }
        })
    }
}

/// Python exceptions with two bases, `FragmentColorError` and a builtin,
/// so an `except` clause for either one catches them.
/// pyo3's `create_exception!` takes one base, so these classes are built at run time.
#[cfg(python)]
#[derive(Clone, Copy)]
pub(crate) enum PythonError {
    /// A scene file that fails to load, also a `RuntimeError`.
    SceneLoad,
    /// A light setter that doesn't fit the light's kind, also a `ValueError`.
    Light,
    /// Camera parameters that leave no usable view, also a `ValueError`.
    Camera,
    /// A material setter value the material can't use, also a `ValueError`.
    Material,
}

#[cfg(python)]
impl PythonError {
    /// The class name, the same one `src/platforms/python.rs` registers it under.
    fn name(self) -> &'static str {
        match self {
            PythonError::SceneLoad => "SceneLoadError",
            PythonError::Light => "LightError",
            PythonError::Camera => "CameraError",
            PythonError::Material => "MaterialError",
        }
    }

    /// The class, built on first use.
    pub(crate) fn class(self, py: Python<'_>) -> PyResult<Bound<'_, pyo3::types::PyType>> {
        use pyo3::exceptions::{PyRuntimeError, PyValueError};
        use pyo3::sync::PyOnceLock;
        use pyo3::types::{PyDict, PyTuple, PyType};

        static SCENE_LOAD: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        static LIGHT: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        static CAMERA: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        static MATERIAL: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        let (cell, builtin) = match self {
            PythonError::SceneLoad => (&SCENE_LOAD, py.get_type::<PyRuntimeError>()),
            PythonError::Light => (&LIGHT, py.get_type::<PyValueError>()),
            PythonError::Camera => (&CAMERA, py.get_type::<PyValueError>()),
            PythonError::Material => (&MATERIAL, py.get_type::<PyValueError>()),
        };
        let class = cell.get_or_try_init(py, || -> PyResult<Py<PyType>> {
            let bases = PyTuple::new(py, [py.get_type::<PyFragmentColorError>(), builtin])?;
            let namespace = PyDict::new(py);
            namespace.set_item("__module__", "fragmentcolor")?;
            let class = py
                .get_type::<PyType>()
                .call1((self.name(), bases, namespace))?;
            Ok(class.cast_into::<PyType>()?.unbind())
        })?;
        Ok(class.bind(py).clone())
    }

    /// An error of this class, or the error that building the class raised.
    pub(crate) fn new_err(self, message: String) -> PyErr {
        Python::attach(|py| match self.class(py) {
            Ok(class) => PyErr::from_type(class, message),
            Err(error) => error,
        })
    }
}

#[cfg(python)]
impl From<PyErr> for crate::target::error::DisplayError {
    fn from(e: PyErr) -> Self {
        crate::target::error::DisplayError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<crate::target::error::DisplayError> for PyErr {
    fn from(e: crate::target::error::DisplayError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for FragmentColorError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        FragmentColorError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<FragmentColorError> for wasm_bindgen::JsValue {
    fn from(error: FragmentColorError) -> Self {
        js_sys::Error::new(&error.to_string()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Top-level error enum wraps inner error kinds via From and displays sensible messages.
    #[test]
    fn wraps_inner_errors_and_displays_messages() {
        // Arrange / Act
        let e1: FragmentColorError =
            crate::shader::error::ShaderError::UniformNotFound("u".into()).into();
        let e2: FragmentColorError =
            crate::color::error::ColorError::TypeMismatch("bad".into()).into();
        let e3: FragmentColorError =
            crate::size::error::SizeError::TypeMismatch("size".into()).into();
        let e6: FragmentColorError = crate::renderer::error::RendererError::NoContext.into();
        let e7: FragmentColorError =
            crate::renderer::error::InitializationError::Error("init".into()).into();
        let e8: FragmentColorError =
            crate::target::error::DisplayError::Error("disp".into()).into();

        #[cfg(not(wasm))]
        let e9: FragmentColorError = {
            let err = crate::net::NetworkError("connection refused".into());
            err.into()
        };

        // Assert
        assert!(e1.to_string().contains("Uniform not found"));
        assert!(e2.to_string().contains("Type mismatch"));
        assert!(e3.to_string().contains("Type mismatch"));
        assert!(e6.to_string().contains("Context not initialized"));
        assert!(e7.to_string().contains("Initialization error"));
        assert!(e8.to_string().contains("Display Error"));
        #[cfg(not(wasm))]
        assert!(e9.to_string().contains("Network"));
    }
}
