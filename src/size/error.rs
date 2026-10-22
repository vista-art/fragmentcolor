use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizeError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM Size Error: {0}")]
    Error(String),
}

#[cfg(python)]
impl From<SizeError> for pyo3::PyErr {
    fn from(error: SizeError) -> Self {
        crate::error::PyFragmentColorError::new_err(error.to_string())
    }
}
