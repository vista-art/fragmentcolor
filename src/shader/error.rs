use thiserror::Error;

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
    WasmError(String),
}

// Python-specific conversions

#[cfg(feature = "python")]
impl From<PyErr> for crate::shader::error::ShaderError {
    fn from(e: PyErr) -> Self {
        crate::shader::error::ShaderError::ParseError(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<crate::shader::error::ShaderError> for PyErr {
    fn from(e: crate::shader::error::ShaderError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
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
        ShaderError::WasmError(error_string)
    }
}

#[cfg(wasm)]
impl From<ShaderError> for wasm_bindgen::JsValue {
    fn from(error: ShaderError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
