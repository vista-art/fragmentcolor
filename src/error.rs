use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(),
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create surface")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("Initialization error: {0}")]
    Error(String),
}

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Failed to parse shader: {0}")]
    ParseError(String),
    #[error("Uniform not found: {0}")]
    UniformNotFound(String),
    #[error("Type mismatch for uniform {0}")]
    TypeMismatch(String),
    #[error("Field not found in struct: {0}")]
    FieldNotFound(String),
    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("WGSL error: {0}")]
    WgslError(#[from] naga::back::wgsl::Error),
    #[error("WGSL Parse error: {0}")]
    WgslParseError(#[from] naga::front::wgsl::ParseError),
    #[error("GLSL Validation error: {0}")]
    GlslValidationError(#[from] naga::WithSpan<naga::valid::ValidationError>),
    #[error("GLSL Parse errors: {0}")]
    GlslParseErrors(#[from] naga::front::glsl::ParseErrors),
    #[error("WGPU error: {0}")]
    WgpuError(#[from] wgpu::Error),
    #[error("WGPU Surface Error: {0}")]
    WgpuSurfaceError(#[from] wgpu::SurfaceError),
    #[error("JSON Deserialization Error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Context Not Initialized")]
    ContextNotInitialized(),

    #[cfg(not(wasm))]
    #[error("URL Request Error: {0}")]
    RequestError(#[from] ureq::Error),
    #[cfg(wasm)]
    #[error("WASM Error: {0}")]
    WasmError(String),
}

// Python-specific conversions

#[cfg(python)]
use pyo3::{create_exception, exceptions::PyException, prelude::*};
#[cfg(python)]
create_exception!(fragment_color, FragmentColorError, PyException);

#[cfg(python)]
impl From<PyErr> for ShaderError {
    fn from(e: PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

#[cfg(python)]
impl From<ShaderError> for PyErr {
    fn from(e: ShaderError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}

#[cfg(python)]
impl From<PyErr> for InitializationError {
    fn from(e: PyErr) -> Self {
        InitializationError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<InitializationError> for PyErr {
    fn from(e: InitializationError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}

// Javascript-specific conversions

#[cfg(wasm)]
use wasm_bindgen::JsError;

#[cfg(wasm)]
impl From<JsError> for ShaderError {
    fn from(_: JsError) -> Self {
        ShaderError::WasmError("JsError".to_string())
    }
}

#[cfg(wasm)]
impl From<JsError> for InitializationError {
    fn from(_: JsError) -> Self {
        InitializationError::Error("JsError".to_string())
    }
}
