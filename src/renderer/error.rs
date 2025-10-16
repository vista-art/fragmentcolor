use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Context not initialized")]
    NoContext,
    #[error("Surface error: failed to acquire frame: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Failed to create texture: {0}")]
    CreateTextureError(String),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::shader::ShaderError),
    #[error("Mesh error: {0}")]
    MeshError(#[from] crate::mesh::MeshError),
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    #[error("Texture error: {0}")]
    TextureError(#[from] crate::texture::TextureError),
    #[error("Texture {0} not found")]
    TextureNotFoundError(crate::texture::TextureId),
    #[error("MSAA texture view missing")]
    MsaaViewMissing,
    #[error("Depth sample_count mismatch: depth={depth} pass={pass}")]
    DepthSampleCountMismatch { depth: u32, pass: u32 },
    #[error("initialization error: {0}")]
    InitializationError(#[from] InitializationError),
    #[cfg(not(wasm))]
    #[error("Network request error: {0}")]
    NetworkRequestError(#[from] ureq::Error),
    #[error("Malformed input error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Renderer error: {0}")]
    Error(String),
}

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Failed to find a compatible GPU adapter")]
    AdapterError(#[from] wgpu::RequestAdapterError),
    #[error("Adapter not set after context()")]
    AdapterNotSet,
    #[error("Failed to create device")]
    DeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create surface")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("Initialization error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
use pyo3::exceptions::PyException as PyFragmentColorError;

#[cfg(python)]
impl From<pyo3::PyErr> for RendererError {
    fn from(e: pyo3::PyErr) -> Self {
        RendererError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<RendererError> for pyo3::PyErr {
    fn from(e: RendererError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

#[cfg(python)]
impl From<pyo3::PyErr> for InitializationError {
    fn from(e: pyo3::PyErr) -> Self {
        InitializationError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<InitializationError> for pyo3::PyErr {
    fn from(e: InitializationError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for RendererError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        RendererError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<RendererError> for wasm_bindgen::JsValue {
    fn from(error: RendererError) -> Self {
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
        InitializationError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<InitializationError> for wasm_bindgen::JsValue {
    fn from(error: InitializationError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: RendererError variants format messages and From conversions work.
    #[test]
    fn renderer_error_variants_and_from() {
        // NoContext
        let e = RendererError::NoContext;
        assert!(e.to_string().contains("Context not initialized"));

        // InitializationError conversion
        let ie = InitializationError::Error("boom".into());
        let e2: RendererError = ie.into();
        assert!(matches!(e2, RendererError::InitializationError(_)));

        // IoError conversion
        let io = std::io::Error::other("oops");
        let e3: RendererError = io.into();
        assert!(matches!(e3, RendererError::IoError(_)));

        // NetworkRequestError (non-WASM)
        #[cfg(not(wasm))]
        {
            let net = ureq::get("http://127.0.0.1:1").call().unwrap_err();
            let e4: RendererError = net.into();
            assert!(matches!(e4, RendererError::NetworkRequestError(_)));
        }
    }

    // Story: SurfaceError variants are wrapped with a helpful message in Display.
    #[test]
    fn surface_error_display_includes_inner() {
        let e = RendererError::SurfaceError(wgpu::SurfaceError::OutOfMemory);
        let s = e.to_string();
        assert!(s.contains("Surface error"));
        assert!(s.contains("failed to acquire frame"));
    }
}
