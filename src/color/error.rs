use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorError {
    /// A color that can't be used, such as an incorrect number of channels.
    #[error("{0}")]
    Invalid(String),
    /// A channel that is NaN or infinite, which no target can clear to.
    #[error("the {channel} channel must be a finite number, got {value}")]
    NotFinite { channel: &'static str, value: f32 },
    /// Text that isn't a CSS color.
    #[error("{text:?} is not a CSS color: {reason}")]
    Css { text: String, reason: String },
}

/// CSS color text the parser refuses, with the parser's error.
pub(crate) struct CssFailure<'text> {
    pub(crate) text: &'text str,
    pub(crate) error: csscolorparser::ParseColorError,
}

/// The parser's reason as text, so `ColorError` holds no parser type.
impl From<CssFailure<'_>> for ColorError {
    fn from(failure: CssFailure<'_>) -> Self {
        ColorError::Css {
            text: failure.text.to_string(),
            reason: failure.error.to_string(),
        }
    }
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
