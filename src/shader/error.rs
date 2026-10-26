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
    #[error("Field not found in struct: {0}")]
    FieldNotFound(String),
    #[error("WGSL error: {0}")]
    WgslError(#[from] naga::back::wgsl::Error),
    #[error("WGSL Parse error: {0}")]
    WgslParseError(#[from] naga::front::wgsl::ParseError),
    /// A type whose layout naga can't compute, such as one past its size limit.
    #[error("Layout error: {0}")]
    LayoutError(#[from] naga::proc::LayoutError),
    #[error("GLSL Validation error: {0}")]
    GlslValidationError(#[from] Box<naga::WithSpan<naga::valid::ValidationError>>),
    #[error("GLSL Parse errors: {0}")]
    GlslParseErrors(#[from] naga::front::glsl::ParseErrors),
    #[cfg(not(wasm))]
    #[error("URL Request Error: {0}")]
    RequestError(#[from] crate::net::NetworkError),
    #[error("File not found: {0}")]
    FileNotFound(#[from] std::io::Error),
    #[error("Index out of bounds for {key}: index {index} >= len {len}")]
    IndexOutOfBounds {
        key: String,
        index: usize,
        len: usize,
    },
    #[error("Shader busy: {0}")]
    Busy(String),
    /// A mesh value that doesn't fit the type its attribute takes.
    #[error("Mesh attribute {name}: expected {expected}, got {found}")]
    AttributeMismatch {
        name: String,
        expected: String,
        found: String,
    },
    /// An attribute one shader typed, which this shader declares another way.
    #[error("Mesh attribute {name} is {fixed} on this mesh, but the shader declares {declared}")]
    AttributeConflict {
        name: String,
        fixed: String,
        declared: String,
    },
    #[cfg(wasm)]
    #[error("WASM Shader Error: {0}")]
    Error(String),
}

/// `error` and each error under it, joined into one line.
pub(crate) fn describe(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        cause = inner.source();
    }
    text
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for ShaderError {
    fn from(e: pyo3::PyErr) -> Self {
        ShaderError::ParseError(e.to_string())
    }
}

#[cfg(python)]
impl From<ShaderError> for pyo3::PyErr {
    fn from(e: ShaderError) -> Self {
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
        js_sys::Error::new(&error.to_string()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: ShaderError formats carry details for various failure kinds.
    #[test]
    fn formats_shader_error_variants() {
        let a = ShaderError::ParseError("p".into());
        let b = ShaderError::PlannedFeature("pf".into());
        let c = ShaderError::UniformNotFound("u".into());
        let d = ShaderError::TypeMismatch("k".into());
        let e = ShaderError::InvalidKey("k".into());
        let f = ShaderError::IndexOutOfBounds {
            key: "arr".into(),
            index: 3,
            len: 2,
        };
        let g = ShaderError::FieldNotFound("x".into());
        let h = ShaderError::AttributeMismatch {
            name: "id".into(),
            expected: "u32".into(),
            found: "-1 at vertex 2".into(),
        };
        let i = ShaderError::AttributeConflict {
            name: "id".into(),
            fixed: "u32".into(),
            declared: "f32".into(),
        };

        assert!(a.to_string().contains("parse"));
        assert!(b.to_string().contains("Planned feature"));
        assert!(c.to_string().contains("Uniform not found"));
        assert!(d.to_string().contains("Type mismatch"));
        assert!(e.to_string().contains("Invalid key"));
        assert!(f.to_string().contains("Index out of bounds"));
        assert!(g.to_string().contains("Field not found"));
        assert_eq!(
            h.to_string(),
            "Mesh attribute id: expected u32, got -1 at vertex 2"
        );
        assert_eq!(
            i.to_string(),
            "Mesh attribute id is u32 on this mesh, but the shader declares f32"
        );
    }
}
