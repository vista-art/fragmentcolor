use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizeError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM Size Error: {0}")]
    Error(String),
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for SizeError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::InvalidInput
    }
}

#[cfg(python)]
impl From<SizeError> for pyo3::PyErr {
    fn from(error: SizeError) -> Self {
        crate::error::raise_error(&error)
    }
}
