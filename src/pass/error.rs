use std::convert::Infallible;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PassError {
    #[error("Invalid uniform root")]
    InvalidUniformRoot,
    #[error("Alias conflict")]
    AliasConflict,
    #[error("No compatible shader exists for this mesh: {0}")]
    NoCompatibleShader(crate::ShaderError),
    #[error("The pass has no shader to attach the mesh to")]
    NoShader,
    #[error("No pass in the scene matches the target: {0}")]
    PassNotFound(String),
    #[error(
        "A shader on this Pass already holds {cap} lights, the most this device allows; drop a Light or give the extra lights another Material"
    )]
    LightCapReached { cap: u32 },
    #[error("Invalid color target: {0}")]
    InvalidColorTarget(String),
    #[error("Invalid clear color: {0}")]
    Color(#[from] crate::color::error::ColorError),
    #[error("Invalid viewport: {0}")]
    Viewport(#[from] crate::region::error::ScreenRegionError),
    #[error("Self dependency is not allowed")]
    SelfDependency,
    #[error("Duplicate dependency: {0}")]
    DuplicateDependency(String),
    #[error("Dependency introduces a cycle via: {via}")]
    DependencyCycle { via: String },
    #[error("Surface error: {0}")]
    SurfaceError(#[from] crate::target::SurfaceError),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::ShaderError),
    #[error("Python Pass Error: {0}")]
    #[cfg(python)]
    Error(String),
    #[cfg(wasm)]
    #[error("WASM Pass Error: {0}")]
    Error(String),
}

/// Lets a conversion that can't fail, such as a `ScreenRegion` passed as itself, use `?`.
impl From<Infallible> for PassError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

// Python-specific conversions

#[cfg(python)]
impl From<pyo3::PyErr> for PassError {
    fn from(e: pyo3::PyErr) -> Self {
        PassError::Error(e.to_string())
    }
}

/// A color or viewport the caller passed raises `InvalidInputError`, and any other error `FragmentColorError`.
#[cfg(python)]
impl From<PassError> for pyo3::PyErr {
    fn from(error: PassError) -> Self {
        match error {
            PassError::Color(_) | PassError::Viewport(_) => {
                crate::error::InvalidInputError::new_err(error.to_string())
            }
            _ => crate::PyFragmentColorError::new_err(error.to_string()),
        }
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
        js_sys::Error::new(&error.to_string()).into()
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
        let c = PassError::NoCompatibleShader(crate::ShaderError::InvalidKey("k".into()));
        let k = PassError::NoShader;
        let d = PassError::InvalidColorTarget("k".into());
        let e = PassError::SelfDependency;
        let f = PassError::DuplicateDependency("p".into());
        let g = PassError::DependencyCycle { via: "x".into() };
        let h = PassError::SurfaceError(crate::target::SurfaceError::Lost);
        let i = PassError::ShaderError(crate::ShaderError::ParseError("p".into()));
        #[cfg(any(python, wasm))]
        let j = PassError::Error("x".into());

        assert!(a.to_string().contains("Invalid uniform root"));
        assert!(b.to_string().contains("Alias conflict"));
        assert_eq!(
            c.to_string(),
            "No compatible shader exists for this mesh: Invalid key: k"
        );
        assert!(k.to_string().contains("no shader"));
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
