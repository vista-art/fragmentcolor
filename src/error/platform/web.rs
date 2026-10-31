//! `WebError`, `defer` and the `JsValue` conversions JavaScript throws.

use crate::FragmentColorError;
use crate::error::{Classify, ErrorClass};

/// The error a web binding throws: a JavaScript `Error` whose `name` is the class Python
/// raises for the same error, or a `TypeError` for an argument of the wrong type, as Python's.
#[doc(hidden)]
pub struct WebError(wasm_bindgen::JsValue);

impl WebError {
    /// An error of `class` with the full `message`.
    pub(crate) fn new(class: ErrorClass, message: &str) -> Self {
        Self::with_fields(class, message, &[])
    }

    /// An error of `class` with the full `message`, carrying each of `fields` as a property.
    pub(crate) fn with_fields(
        class: ErrorClass,
        message: &str,
        fields: &[(&str, crate::core::Field)],
    ) -> Self {
        use crate::core::Field;
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

impl From<WebError> for wasm_bindgen::JsValue {
    fn from(error: WebError) -> Self {
        error.0
    }
}

/// The promise a web export returns for `future`. It rejects only with a `WebError`, so every
/// rejection is an `Error` named after its class, and any other error type fails to compile.
pub(crate) fn defer(
    future: impl std::future::Future<Output = Result<wasm_bindgen::JsValue, WebError>> + 'static,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move { Ok(future.await?) })
}

/// Each error type that throws as an `Error` named after its class, with no fields.
macro_rules! throw_by_class {
    ($($error:ty),* $(,)?) => {$(
        impl From<$error> for WebError {
            fn from(error: $error) -> Self {
                WebError::from_error(&error)
            }
        }
    )*};
}

throw_by_class!(
    FragmentColorError,
    crate::ShaderError,
    crate::ColorError,
    crate::SizeError,
    crate::PassError,
    crate::InitializationError,
    crate::DisplayError,
    crate::MeshError,
    crate::ScreenRegionError,
    crate::bindings::numbers::VectorError,
    crate::MaterialError,
    crate::scene::LightError,
    crate::scene::CameraError,
    crate::scene::SceneLoadError,
);

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

impl From<FragmentColorError> for wasm_bindgen::JsValue {
    fn from(error: FragmentColorError) -> Self {
        WebError::from(error).into()
    }
}
