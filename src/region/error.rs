use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM ScreenRegion Error: {0}")]
    Error(String),
}

#[cfg(python)]
impl From<ScreenRegionError> for pyo3::PyErr {
    fn from(error: ScreenRegionError) -> Self {
        crate::error::PyFragmentColorError::new_err(error.to_string())
    }
}
