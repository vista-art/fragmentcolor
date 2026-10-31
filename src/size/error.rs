use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizeError {
    /// A size a binding can't use, such as a wrong count of numbers or a negative one, which the message names.
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
