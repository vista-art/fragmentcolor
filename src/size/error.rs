use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizeError {
    /// A size that can't be used, such as an incorrect number of values or a negative one.
    #[error("{0}")]
    Invalid(String),
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
