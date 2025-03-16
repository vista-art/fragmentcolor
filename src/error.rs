use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use thiserror::Error;

create_exception!(fragment_color, FragmentColorError, PyException);

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(),
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create surface")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),
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
    #[error("URL Request Error: {0}")]
    RequestError(#[from] ureq::Error),
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
}

impl From<PyErr> for ShaderError {
    fn from(e: PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

impl From<ShaderError> for PyErr {
    fn from(e: ShaderError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}

impl From<InitializationError> for PyErr {
    fn from(e: InitializationError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}
