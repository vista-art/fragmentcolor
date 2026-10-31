//! The Python exception classes, `raise` and `PythonError`.

use crate::core::Field;
use crate::error::{Classify, ErrorClass};
use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyException},
    prelude::*,
};

create_exception!(fragment_color, PyFragmentColorError, PyException);
create_exception!(
    fragment_color,
    InvalidInputError,
    PyFragmentColorError,
    "A value the call can't take, such as a negative size or bytes that don't match their texture."
);
create_exception!(
    fragment_color,
    UnsupportedMipmapFormatError,
    PyFragmentColorError,
    "A texture format the CPU mipmap builder can't fill."
);
create_exception!(
    fragment_color,
    MalformedImageError,
    PyFragmentColorError,
    "Image bytes that can't be decoded."
);
create_exception!(
    fragment_color,
    OccludedSurfaceError,
    PyFragmentColorError,
    "A window that is hidden or minimized, so render skips the frame."
);
create_exception!(
    fragment_color,
    DeviceError,
    PyFragmentColorError,
    "A GPU error an error scope caught, with its kind, the label of the step that raised it and the device's own detail."
);

create_exception!(
    fragment_color,
    LimitError,
    PyFragmentColorError,
    "Something in a shader or mesh past one of the device's limits, with the item, the limit's name, the value it needs and the device's maximum."
);
create_exception!(
    fragment_color,
    FeatureError,
    PyFragmentColorError,
    "Something in a shader that needs a feature the device lacks, with the item and the feature."
);
create_exception!(
    fragment_color,
    BindingError,
    PyFragmentColorError,
    "A texture uniform whose slot can't take its texture, or that was never set, with the uniform's name, what the slot takes and what it got."
);
create_exception!(
    fragment_color,
    EntryError,
    PyFragmentColorError,
    "A shader without the entry point its pass runs, with the stage."
);

/// An exception of class `T` with the full `message`, carrying the attributes `fields` sets.
pub(crate) fn raise<T: pyo3::PyTypeInfo>(
    message: String,
    fields: impl FnOnce(&Bound<'_, PyBaseException>) -> PyResult<()>,
) -> PyErr {
    Python::attach(|py| {
        let error = PyErr::new::<T, _>(message);
        match fields(error.value(py)) {
            Ok(()) => error,
            Err(failure) => failure,
        }
    })
}

/// An exception of `class` with the full `message`.
pub(crate) fn raise_class(class: ErrorClass, message: String) -> PyErr {
    match class {
        ErrorClass::Base => PyFragmentColorError::new_err(message),
        ErrorClass::InvalidInput => InvalidInputError::new_err(message),
        ErrorClass::UnsupportedMipmapFormat => UnsupportedMipmapFormatError::new_err(message),
        ErrorClass::MalformedImage => MalformedImageError::new_err(message),
        ErrorClass::OccludedSurface => OccludedSurfaceError::new_err(message),
        ErrorClass::Device => DeviceError::new_err(message),
        ErrorClass::Limit => LimitError::new_err(message),
        ErrorClass::Feature => FeatureError::new_err(message),
        ErrorClass::Binding => BindingError::new_err(message),
        ErrorClass::Entry => EntryError::new_err(message),
        ErrorClass::SceneLoad => PythonError::SceneLoad.new_err(message),
        ErrorClass::Light => PythonError::Light.new_err(message),
        ErrorClass::Camera => PythonError::Camera.new_err(message),
        ErrorClass::Material => PythonError::Material.new_err(message),
    }
}

/// The exception for `error`, of the class it names.
pub(crate) fn raise_error<E: Classify + std::fmt::Display>(error: &E) -> PyErr {
    raise_class(error.classify(), error.to_string())
}

/// An exception of `class` with the full `message` that carries each of `fields` as an attribute.
pub(crate) fn raise_fields(
    class: ErrorClass,
    message: String,
    fields: Vec<(&'static str, Field)>,
) -> PyErr {
    let error = raise_class(class, message);
    Python::attach(|py| {
        let value = error.value(py);
        for (name, field) in fields {
            let set = match field {
                Field::Text(text) => value.setattr(name, text),
                Field::Number(number) => value.setattr(name, number),
            };
            if let Err(failure) = set {
                return failure;
            }
        }
        error
    })
}

impl DeviceError {
    /// A `DeviceError` with the full `message` that carries the GPU error's `kind`,
    /// the `label` of its step and the device's own `detail`.
    pub(crate) fn raise(
        kind: crate::ErrorKind,
        label: String,
        detail: String,
        message: String,
    ) -> PyErr {
        raise::<DeviceError>(message, |error| {
            error.setattr("kind", kind)?;
            error.setattr("label", label)?;
            error.setattr("detail", detail)
        })
    }
}

/// Python exceptions with two bases, `FragmentColorError` and a builtin,
/// so an `except` clause for either one catches them.
/// pyo3's `create_exception!` takes one base, so these classes are built at run time.
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

impl PythonError {
    /// The class name, the same one `src/bindings/python.rs` registers it under.
    fn name(self) -> &'static str {
        let class = match self {
            PythonError::SceneLoad => ErrorClass::SceneLoad,
            PythonError::Light => ErrorClass::Light,
            PythonError::Camera => ErrorClass::Camera,
            PythonError::Material => ErrorClass::Material,
        };
        class.name()
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

impl From<PyErr> for crate::DisplayError {
    fn from(e: PyErr) -> Self {
        crate::DisplayError::Error(e.to_string())
    }
}

impl From<crate::DisplayError> for PyErr {
    fn from(e: crate::DisplayError) -> Self {
        raise_error(&e)
    }
}
