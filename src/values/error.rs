//! The errors `Size`, `ScreenRegion` and `Color` refuse their input with.

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

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    /// A region that can't be used, such as an incorrect number of values or a negative one.
    #[error("{0}")]
    Invalid(String),
    /// A region with no pixels, which no viewport or scissor can use.
    #[error("a region needs a width and a height of at least 1, got {width} by {height}")]
    Empty { width: u32, height: u32 },
    /// A region whose max lies before its min on an axis.
    #[error(
        "a region needs each max past its min, got x from {min_x} to {max_x} and y from {min_y} to {max_y}"
    )]
    Reversed {
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
    },
    /// A region whose far edge would pass `u32::MAX`, which no pixel coordinate can hold.
    #[error(
        "a region at {x}, {y} sized {width} by {height} reaches past {}",
        u32::MAX
    )]
    Overflow {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for ScreenRegionError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::InvalidInput
    }
}

#[cfg(python)]
impl From<ScreenRegionError> for pyo3::PyErr {
    fn from(error: ScreenRegionError) -> Self {
        crate::error::raise_error(&error)
    }
}

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

/// CSS color text the parser refuses.
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
