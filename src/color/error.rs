use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorError {
    /// A color a binding can't use, such as a wrong count of channels, which the message names.
    #[error("{0}")]
    Invalid(String),
    /// A channel that is NaN or infinite, which no target can clear to.
    #[error("the {channel} channel must be a finite number, got {value}")]
    NotFinite { channel: &'static str, value: f32 },
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for ColorError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::InvalidInput
    }
}

#[cfg(python)]
impl From<ColorError> for pyo3::PyErr {
    fn from(error: ColorError) -> Self {
        crate::error::raise_error(&error)
    }
}
