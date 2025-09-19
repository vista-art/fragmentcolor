use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Context not initialized")]
    NoContext,
    #[error("Image error: {0}")]
    MalformedImageError(#[from] image::ImageError),
    #[error("Failed to create texture: {0}")]
    CreateTextureError(String),
    #[error("Shader error: {0}")]
    ShaderError(#[from] crate::shader::error::ShaderError),
    #[error("Bind Group Layout error: {0}")]
    BindGroupLayoutError(String),
    #[error("Renderer error: {0}")]
    Error(String),
}

// Python-specific conversions

#[cfg(python)]
use pyo3::exceptions::PyException as PyFragmentColorError;

#[cfg(python)]
impl From<pyo3::PyErr> for TextureError {
    fn from(e: pyo3::PyErr) -> Self {
        TextureError::Error(e.to_string())
    }
}

#[cfg(python)]
impl From<TextureError> for pyo3::PyErr {
    fn from(e: TextureError) -> Self {
        PyFragmentColorError::new_err(e.to_string())
    }
}

// WASM-specific conversions

#[cfg(wasm)]
impl From<wasm_bindgen::JsValue> for TextureError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let error_string = if let Some(s) = value.as_string() {
            s
        } else {
            format!("{:?}", value)
        };
        TextureError::Error(error_string)
    }
}

#[cfg(wasm)]
impl From<TextureError> for wasm_bindgen::JsValue {
    fn from(error: TextureError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: TextureError wraps failures from creation paths and shader conversion.
    #[test]
    fn texture_error_variants_and_from_shader() {
        let e1 = TextureError::NoContext;
        assert!(e1.to_string().contains("Context not initialized"));

        let e2 = TextureError::CreateTextureError("bad".into());
        assert!(e2.to_string().contains("Failed to create texture"));

        // From ShaderError
        let se = crate::shader::error::ShaderError::UniformNotFound("u".into());
        let e3: TextureError = se.into();
        assert!(matches!(e3, TextureError::ShaderError(_)));
    }
}
