use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(#[from] wgpu::RequestAdapterError),
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create surface")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("Initialization error: {0}")]
    Error(String),
    #[cfg(wasm)]
    #[error("WASM Initialization Error: {0}")]
    WasmError(String),
}

#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Failed to create a window handle: {0}")]
    WindowHandleError(String),
    #[error("Failed to create a display handle: {0}")]
    DisplayHandleError(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Display Error: {0}")]
    Error(String),
    #[cfg(wasm)]
    #[error("WASM Display Error: {0}")]
    WasmError(String),
}

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Context not initialized")]
    NoContext(),
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
    GlslValidationError(#[from] Box<naga::WithSpan<naga::valid::ValidationError>>),
    #[error("GLSL Parse errors: {0}")]
    GlslParseErrors(#[from] naga::front::glsl::ParseErrors),
    #[error("WGPU error: {0}")]
    WgpuError(#[from] wgpu::Error),
    #[error("WGPU Surface Error: {0}")]
    WgpuSurfaceError(#[from] wgpu::SurfaceError),

    #[cfg(not(wasm))]
    #[error("URL Request Error: {0}")]
    RequestError(#[from] ureq::Error),
    #[cfg(wasm)]
    #[error("WASM Shader Error: {0}")]
    WasmError(String),
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for ShaderError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        ShaderError::WasmError(error_string)
    }
}

#[cfg(wasm)]
impl From<ShaderError> for wasm_bindgen::JsValue {
    fn from(error: ShaderError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for DisplayError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        DisplayError::WasmError(error_string)
    }
}

#[cfg(wasm)]
impl From<DisplayError> for wasm_bindgen::JsValue {
    fn from(error: DisplayError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for InitializationError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        InitializationError::WasmError(error_string)
    }
}

#[cfg(wasm)]
impl From<InitializationError> for wasm_bindgen::JsValue {
    fn from(error: InitializationError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

// Python-specific conversions

#[cfg(feature = "python")]
use pyo3::{create_exception, exceptions::PyException, prelude::*};
#[cfg(feature = "python")]
create_exception!(fragment_color, FragmentColorError, PyException);

#[cfg(feature = "python")]
impl From<PyErr> for ShaderError {
    fn from(e: PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<ShaderError> for PyErr {
    fn from(e: ShaderError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<PyErr> for DisplayError {
    fn from(e: PyErr) -> Self {
        DisplayError::Error(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<DisplayError> for PyErr {
    fn from(e: DisplayError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<PyErr> for InitializationError {
    fn from(e: PyErr) -> Self {
        InitializationError::Error(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<InitializationError> for PyErr {
    fn from(e: InitializationError) -> Self {
        FragmentColorError::new_err(e.to_string())
    }
}
