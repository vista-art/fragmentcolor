use std::convert::Infallible;
use thiserror::Error;

use super::PassType;

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
    #[error("A {pass} pass can't take a {shader} shader")]
    ShaderKind { pass: PassType, shader: PassType },
    #[error("No pass in the scene matches the target: {0}")]
    PassNotFound(String),
    #[error(
        "A shader on this Pass takes at most {cap} lights on this device; drop a Light or give the extra lights another Material"
    )]
    LightCapReached { cap: u32 },
    #[error("Invalid color target: {0}")]
    InvalidColorTarget(String),
    /// A compute dispatch with more workgroups in one dimension than the device takes.
    #[error(
        "Pass '{pass}' dispatches {dispatch:?} workgroups, and this device takes at most {limit} in each dimension"
    )]
    DispatchLimit {
        pass: String,
        dispatch: [u32; 3],
        limit: u32,
    },
    /// A viewport with a side past the device's 2D texture limit.
    #[error(
        "Pass '{pass}' has a viewport of {}x{}, and this device takes at most {limit} on each side",
        .size[0], .size[1]
    )]
    ViewportLimit {
        pass: String,
        size: [u32; 2],
        limit: u32,
    },
    /// A texture one pass both writes and reads, or writes in two ways.
    #[error("Texture conflict in pass '{pass}': {detail}")]
    TextureConflict { pass: String, detail: String },
    #[error("Invalid clear color: {0}")]
    Color(#[from] crate::ColorError),
    #[error("Invalid viewport: {0}")]
    Viewport(#[from] crate::ScreenRegionError),
    #[error("Self dependency is not allowed")]
    SelfDependency,
    #[error("Duplicate dependency: {0}")]
    DuplicateDependency(String),
    #[error("Dependency introduces a cycle via: {via}")]
    DependencyCycle { via: String },
    #[error("Surface error: {0}")]
    SurfaceError(#[from] crate::SurfaceError),
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

/// A value the caller passed is an `InvalidInputError`, whether a setter or `render`
/// refuses it: a color, a viewport, a target texture or a dispatch. Any other error
/// is a `FragmentColorError`.
#[cfg(any(python, wasm, test))]
impl crate::error::Classify for PassError {
    fn classify(&self) -> crate::error::ErrorClass {
        use crate::error::ErrorClass;
        match self {
            PassError::Color(_)
            | PassError::Viewport(_)
            | PassError::ViewportLimit { .. }
            | PassError::InvalidColorTarget(_)
            | PassError::DispatchLimit { .. } => ErrorClass::InvalidInput,
            _ => ErrorClass::Base,
        }
    }
}

#[cfg(python)]
impl From<PassError> for pyo3::PyErr {
    fn from(error: PassError) -> Self {
        crate::error::raise_error(&error)
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
        crate::error::WebError::from(error).into()
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
        let kind = PassError::ShaderKind {
            pass: PassType::Compute,
            shader: PassType::Render,
        };
        let d = PassError::InvalidColorTarget("k".into());
        let e = PassError::SelfDependency;
        let f = PassError::DuplicateDependency("p".into());
        let g = PassError::DependencyCycle { via: "x".into() };
        let h = PassError::SurfaceError(crate::SurfaceError::Lost);
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
        assert_eq!(
            kind.to_string(),
            "A compute pass can't take a render shader"
        );
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
