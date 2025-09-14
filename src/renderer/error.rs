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
    ShaderError(#[from] crate::shader::error::ShaderError),
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    #[error("Renderer error: {0}")]
    Error(String),
    #[cfg(wasm)]
    #[error("WASM Renderer Error: {0}")]
    WasmError(String),
}

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

// Python-specific conversions

#[cfg(feature = "python")]
impl From<PyErr> for crate::renderer::error::RendererError {
    fn from(e: PyErr) -> Self {
        crate::renderer::error::RendererError::Error(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<crate::renderer::error::RendererError> for PyErr {
    fn from(e: crate::renderer::error::RendererError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<PyErr> for crate::renderer::error::InitializationError {
    fn from(e: PyErr) -> Self {
        crate::renderer::error::InitializationError::Error(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<crate::renderer::error::InitializationError> for PyErr {
    fn from(e: crate::renderer::error::InitializationError) -> Self {
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
        RendererError::WasmError(error_string)
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
        InitializationError::WasmError(error_string)
    }
}

#[cfg(wasm)]
impl From<InitializationError> for wasm_bindgen::JsValue {
    fn from(error: InitializationError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
