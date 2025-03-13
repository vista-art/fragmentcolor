use pyo3::{prelude::*, PyErrArguments};
use thiserror::Error;

#[pyclass]
#[derive(Error, Debug)]
pub enum FragmentColorError {
    #[error("Internal Initialization error: {0}")]
    InitializationError(String),
    #[error("Internal Shader error: {0}")]
    ShaderError(String),
}

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(),
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
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

impl From<InitializationError> for FragmentColorError {
    fn from(e: InitializationError) -> Self {
        FragmentColorError::InitializationError(e.to_string())
    }
}

impl From<ShaderError> for FragmentColorError {
    fn from(e: ShaderError) -> Self {
        FragmentColorError::ShaderError(e.to_string())
    }
}

impl From<PyErr> for FragmentColorError {
    fn from(e: PyErr) -> Self {
        FragmentColorError::InitializationError(e.to_string())
    }
}

impl From<FragmentColorError> for PyErr {
    fn from(e: FragmentColorError) -> Self {
        PyErr::new::<FragmentColorError, _>(e)
    }
}

impl From<PyErr> for ShaderError {
    fn from(e: PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

impl From<ShaderError> for PyErr {
    fn from(e: ShaderError) -> Self {
        PyErr::new::<FragmentColorError, _>(e)
    }
}

impl From<InitializationError> for PyErr {
    fn from(e: InitializationError) -> Self {
        PyErr::new::<FragmentColorError, _>(e)
    }
}

impl PyErrArguments for InitializationError {
    fn arguments(self, py: Python<'_>) -> PyObject {
        let pyerr: PyErr = FragmentColorError::InitializationError(self.to_string()).into();
        pyerr.into_pyobject(py).unwrap().into()
    }
}

impl PyErrArguments for ShaderError {
    fn arguments(self, py: Python<'_>) -> PyObject {
        let pyerr: PyErr = FragmentColorError::ShaderError(self.to_string()).into();
        pyerr.into_pyobject(py).unwrap().into()
    }
}
