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
    Frame(#[from] crate::frame::error::FrameError),
    #[error(transparent)]
    Renderer(#[from] crate::renderer::error::RendererError),
    #[error(transparent)]
    Init(#[from] crate::renderer::error::InitializationError),
    #[error(transparent)]
    Display(#[from] crate::target::error::DisplayError),
    #[cfg(not(wasm))]
    #[error("Network Request Error: {0}")]
    NetworkRequest(#[from] ureq::Error),
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
        wasm_bindgen::JsValue::from_str(&error.to_string())
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
            let err = ureq::get("http://127.0.0.1:1").call().unwrap_err();
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
