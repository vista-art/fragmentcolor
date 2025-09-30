use thiserror::Error;

#[derive(Error, Debug)]
pub enum PassError {
    #[error("Invalid uniform root")]
    InvalidUniformRoot,
    #[error("Alias conflict")]
    AliasConflict,
    #[error("No compatible shader exists for this mesh")]
    NoCompatibleShader,
    #[error("Invalid color target: {0}")]
    InvalidColorTarget(String),
    #[error("Self dependency is not allowed")]
    SelfDependency,
    #[error("Duplicate dependency: {0}")]
    DuplicateDependency(String),
    #[error("Dependency introduces a cycle via: {via}")]
    DependencyCycle { via: String },
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::ShaderError),
    #[error("Python Pass Error: {0}")]
    #[cfg(python)]
    Error(String),
    #[cfg(wasm)]
    #[error("WASM Pass Error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for PassError {
    fn from(e: pyo3::PyErr) -> Self {
        PassError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<PassError> for pyo3::PyErr {
    fn from(e: PassError) -> Self {
        crate::PyFragmentColorError::new_err(e.to_string())
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for PassError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        PassError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<PassError> for wasm_bindgen::JsValue {
    fn from(error: PassError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: PassError formats carry details for various failure kinds.
    #[test]
    fn formats_shader_error_variants() {
        let a = PassError::InvalidUniformRoot;
        let b = PassError::AliasConflict;
        let c = PassError::NoCompatibleShader;
        let d = PassError::InvalidColorTarget("k".into());
        let e = PassError::SelfDependency;
        let f = PassError::DuplicateDependency("p".into());
        let g = PassError::DependencyCycle { via: "x".into() };
        let h = PassError::SurfaceError(wgpu::SurfaceError::Lost);
        let i = PassError::ShaderError(crate::ShaderError::ParseError("p".into()));
        #[cfg(any(python, wasm))]
        let j = PassError::Error("x".into());

        assert!(a.to_string().contains("Invalid uniform root"));
        assert!(b.to_string().contains("Alias conflict"));
        assert!(c.to_string().contains("No compatible shader"));
        assert!(d.to_string().contains("Invalid color target"));
        assert!(e.to_string().contains("Self dependency"));
        assert!(f.to_string().contains("Duplicate dependency"));
        assert!(g.to_string().contains("cycle"));
        assert!(h.to_string().contains("Surface error"));
        assert!(i.to_string().contains("Shader error"));
        #[cfg(any(python, wasm))]
        assert!(j.to_string().contains("Error"));
    }
}
