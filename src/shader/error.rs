use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Failed to parse shader: {0}")]
    ParseError(String),
    #[error("Planned feature not yet implemented: {0}")]
    PlannedFeature(String),
    #[error("Uniform not found: {0}")]
    UniformNotFound(String),
    #[error("Type mismatch for uniform {0}")]
    TypeMismatch(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Index out of bounds for {key}: index {index} >= len {len}")]
    IndexOutOfBounds { key: String, index: usize, len: usize },
    #[error("Field not found in struct: {0}")]
    FieldNotFound(String),
    #[error("WGSL error: {0}")]
    WgslError(#[from] naga::back::wgsl::Error),
    #[error("WGSL Parse error: {0}")]
    WgslParseError(#[from] naga::front::wgsl::ParseError),
    #[error("GLSL Validation error: {0}")]
    GlslValidationError(#[from] Box<naga::WithSpan<naga::valid::ValidationError>>),
    #[error("GLSL Parse errors: {0}")]
    GlslParseErrors(#[from] naga::front::glsl::ParseErrors),
    #[cfg(not(wasm))]
    #[error("URL Request Error: {0}")]
    RequestError(#[from] ureq::Error),
    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[cfg(wasm)]
    #[error("WASM Shader Error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for crate::shader::error::ShaderError {
    fn from(e: pyo3::PyErr) -> Self {
        crate::shader::error::ShaderError::ParseError(e.to_string())
    }
}

#[cfg(python)]
impl From<crate::shader::error::ShaderError> for pyo3::PyErr {
    fn from(e: crate::shader::error::ShaderError) -> Self {
        crate::PyFragmentColorError::new_err(e.to_string())
    }
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
        ShaderError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<ShaderError> for wasm_bindgen::JsValue {
    fn from(error: ShaderError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
