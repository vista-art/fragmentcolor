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

/// The class of error each binding raises, named after the Python exception class.
/// Python raises that class, and JavaScript throws an `Error` with that `name`.
#[cfg(any(python, wasm, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorClass {
    /// `FragmentColorError`, the class every other one extends.
    Base,
    InvalidInput,
    UnsupportedMipmapFormat,
    MalformedImage,
    OccludedSurface,
    Device,
    Limit,
    Feature,
    Binding,
    Entry,
    SceneLoad,
    Light,
    Camera,
    Material,
}

#[cfg(any(python, wasm, test))]
impl ErrorClass {
    /// The class name, the same in Python and JavaScript.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ErrorClass::Base => "FragmentColorError",
            ErrorClass::InvalidInput => "InvalidInputError",
            ErrorClass::UnsupportedMipmapFormat => "UnsupportedMipmapFormatError",
            ErrorClass::MalformedImage => "MalformedImageError",
            ErrorClass::OccludedSurface => "OccludedSurfaceError",
            ErrorClass::Device => "DeviceError",
            ErrorClass::Limit => "LimitError",
            ErrorClass::Feature => "FeatureError",
            ErrorClass::Binding => "BindingError",
            ErrorClass::Entry => "EntryError",
            ErrorClass::SceneLoad => "SceneLoadError",
            ErrorClass::Light => "LightError",
            ErrorClass::Camera => "CameraError",
            ErrorClass::Material => "MaterialError",
        }
    }
}

/// An error that knows the class every binding raises it as.
#[cfg(any(python, wasm, test))]
pub(crate) trait Classify {
    /// The class Python raises this error as, and the `name` of the JavaScript `Error`.
    fn classify(&self) -> ErrorClass;
}

#[cfg(any(python, wasm, test))]
impl Classify for FragmentColorError {
    fn classify(&self) -> ErrorClass {
        match self {
            FragmentColorError::Shader(error) => error.classify(),
            FragmentColorError::Color(error) => error.classify(),
            FragmentColorError::Size(error) => error.classify(),
            FragmentColorError::Pass(error) => error.classify(),
            FragmentColorError::Renderer(error) => error.classify(),
            FragmentColorError::Init(error) => error.classify(),
            FragmentColorError::Display(error) => error.classify(),
            #[cfg(not(wasm))]
            FragmentColorError::NetworkRequest(_) => ErrorClass::Base,
            #[cfg(wasm)]
            FragmentColorError::Error(_) => ErrorClass::Base,
        }
    }
}

/// The error a web binding throws: a JavaScript `Error` whose `name` is the class Python
/// raises for the same error, or a `TypeError` for an argument of the wrong type, as Python's.
#[cfg(wasm)]
#[doc(hidden)]
pub struct WebError(wasm_bindgen::JsValue);

#[cfg(wasm)]
impl WebError {
    /// An error of `class` with the full `message`.
    pub(crate) fn new(class: ErrorClass, message: &str) -> Self {
        Self::with_fields(class, message, &[])
    }

    /// An error of `class` with the full `message`, carrying each of `fields` as a property.
    pub(crate) fn with_fields(
        class: ErrorClass,
        message: &str,
        fields: &[(&str, crate::renderer::error::Field)],
    ) -> Self {
        use crate::renderer::error::Field;
        use wasm_bindgen::JsValue;
        let error = js_sys::Error::new(message);
        error.set_name(class.name());
        let error: JsValue = error.into();
        for (name, field) in fields {
            let field = match field {
                Field::Text(text) => JsValue::from_str(text),
                Field::Number(number) => JsValue::from_f64(*number as f64),
            };
            // Setting a property on a new Error object never throws.
            let _ = js_sys::Reflect::set(&error, &JsValue::from_str(name), &field);
        }
        Self(error)
    }

    /// The error for `error`, named after its class.
    pub(crate) fn from_error<E: Classify + std::fmt::Display>(error: &E) -> Self {
        Self::new(error.classify(), &error.to_string())
    }

    /// A `TypeError` for an argument of the wrong type, as Python raises one.
    pub(crate) fn type_error(message: &str) -> Self {
        WebError(js_sys::TypeError::new(message).into())
    }
}

#[cfg(wasm)]
impl From<WebError> for wasm_bindgen::JsValue {
    fn from(error: WebError) -> Self {
        error.0
    }
}

/// The promise a web export returns for `future`. It rejects only with a `WebError`, so every
/// rejection is an `Error` named after its class, and any other error type fails to compile.
#[cfg(wasm)]
pub(crate) fn defer(
    future: impl std::future::Future<Output = Result<wasm_bindgen::JsValue, WebError>> + 'static,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move { Ok(future.await?) })
}

/// Each error type that throws as an `Error` named after its class, with no fields.
#[cfg(wasm)]
macro_rules! throw_by_class {
    ($($error:ty),* $(,)?) => {$(
        impl From<$error> for WebError {
            fn from(error: $error) -> Self {
                WebError::from_error(&error)
            }
        }
    )*};
}

#[cfg(wasm)]
throw_by_class!(
    FragmentColorError,
    crate::shader::error::ShaderError,
    crate::color::error::ColorError,
    crate::size::error::SizeError,
    crate::pass::error::PassError,
    crate::renderer::error::InitializationError,
    crate::target::error::DisplayError,
    crate::mesh::MeshError,
    crate::region::error::ScreenRegionError,
    crate::platforms::vector::VectorError,
    crate::material::MaterialError,
    crate::scene::LightError,
    crate::scene::CameraError,
    crate::scene::SceneLoadError,
);

// Python-specific conversions

#[cfg(python)]
use crate::renderer::error::Field;
#[cfg(python)]
use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyException},
    prelude::*,
};
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
create_exception!(
    fragment_color,
    LimitError,
    PyFragmentColorError,
    "Something in a shader or mesh past one of the device's limits, with the item, the limit's name, the value it needs and the device's maximum."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    FeatureError,
    PyFragmentColorError,
    "Something in a shader that needs a feature the device lacks, with the item and the feature."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    BindingError,
    PyFragmentColorError,
    "A texture uniform whose slot can't take its texture, or that was never set, with the uniform's name, what the slot takes and what it got."
);
#[cfg(python)]
create_exception!(
    fragment_color,
    EntryError,
    PyFragmentColorError,
    "A shader without the entry point its pass runs, with the stage."
);

/// An exception of class `T` with the full `message`, carrying the attributes `fields` sets.
#[cfg(python)]
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
#[cfg(python)]
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
#[cfg(python)]
pub(crate) fn raise_error<E: Classify + std::fmt::Display>(error: &E) -> PyErr {
    raise_class(error.classify(), error.to_string())
}

/// An exception of `class` with the full `message` that carries each of `fields` as an attribute.
#[cfg(python)]
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

#[cfg(python)]
impl From<PyErr> for crate::target::error::DisplayError {
    fn from(e: PyErr) -> Self {
        crate::target::error::DisplayError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<crate::target::error::DisplayError> for PyErr {
    fn from(e: crate::target::error::DisplayError) -> Self {
        raise_error(&e)
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
        WebError::from(error).into()
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
            crate::color::error::ColorError::Invalid("a color takes 3 or 4 channels".into()).into();
        let e3: FragmentColorError =
            crate::size::error::SizeError::Invalid("a size takes 2 or 3 numbers".into()).into();
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
        assert_eq!(e2.to_string(), "a color takes 3 or 4 channels");
        assert_eq!(e3.to_string(), "a size takes 2 or 3 numbers");
        assert!(e6.to_string().contains("Context not initialized"));
        assert!(e7.to_string().contains("Initialization error"));
        assert!(e8.to_string().contains("Display Error"));
        #[cfg(not(wasm))]
        assert!(e9.to_string().contains("Network"));
    }

    // Story: each error takes the class Python raises it as, which is also the `name` of the
    // Error JavaScript throws, and an error that wraps another takes the class of what it wraps.
    #[test]
    fn errors_take_the_python_class() {
        use crate::color::error::ColorError;
        use crate::pass::error::PassError;
        use crate::region::error::ScreenRegionError;
        use crate::renderer::error::{InitializationError, RendererError};
        use crate::scene::{LightError, SceneLoadError};
        use crate::shader::error::ShaderError;
        use crate::target::SurfaceError;
        use crate::texture::TextureError;
        let text = |value: &str| value.to_string();
        let limit = RendererError::LimitError {
            item: text("group 9"),
            limit: text("max_bind_groups"),
            value: 10,
            max: 4,
        };
        let cases: Vec<(Box<dyn Classify>, &str)> = vec![
            (
                Box::new(ShaderError::ParseError(text("x"))),
                "FragmentColorError",
            ),
            (Box::new(RendererError::NoContext), "FragmentColorError"),
            (
                Box::new(RendererError::SurfaceError(SurfaceError::Lost)),
                "FragmentColorError",
            ),
            (
                Box::new(RendererError::SurfaceError(SurfaceError::Occluded)),
                "OccludedSurfaceError",
            ),
            (
                Box::new(RendererError::DeviceError {
                    kind: crate::ErrorKind::Validation,
                    label: text("frame"),
                    detail: text("x"),
                }),
                "DeviceError",
            ),
            (
                Box::new(RendererError::FeatureError {
                    item: text("the shader"),
                    feature: text("x"),
                }),
                "FeatureError",
            ),
            (
                Box::new(RendererError::BindingError {
                    uniform: text("image"),
                    expected: text("a texture"),
                    found: text("none"),
                }),
                "BindingError",
            ),
            (
                Box::new(RendererError::EntryError {
                    stage: text("@vertex"),
                }),
                "EntryError",
            ),
            (
                Box::new(RendererError::TextureError(TextureError::InvalidInput(
                    text("x"),
                ))),
                "InvalidInputError",
            ),
            (
                Box::new(RendererError::PassError(PassError::Viewport(
                    ScreenRegionError::Empty {
                        width: 0,
                        height: 4,
                    },
                ))),
                "InvalidInputError",
            ),
            (
                Box::new(RendererError::InitializationError(
                    InitializationError::TextureError(TextureError::MalformedImageError(text("x"))),
                )),
                "MalformedImageError",
            ),
            (
                Box::new(TextureError::UnsupportedMipmapFormat {
                    format: crate::TextureFormat::R8Unorm,
                }),
                "UnsupportedMipmapFormatError",
            ),
            (
                Box::new(ColorError::Invalid(text("x"))),
                "InvalidInputError",
            ),
            (
                Box::new(SceneLoadError::Invalid(text("x"))),
                "SceneLoadError",
            ),
            (Box::new(LightError::NegativeRange(-1.0)), "LightError"),
            (Box::new(FragmentColorError::Renderer(limit)), "LimitError"),
        ];

        for (error, name) in cases {
            assert_eq!(error.classify().name(), name);
        }
    }
}
