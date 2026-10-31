use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    /// A region a binding can't use, such as a wrong count of numbers or a negative one, which the message names.
    #[error("{0}")]
    Invalid(String),
    /// A region with no pixels, which no viewport or scissor can use.
    #[error("a region needs a width and a height of at least 1, got {width} by {height}")]
    Empty { width: u32, height: u32 },
    /// A region whose max lies before its min on an axis, which no binding swaps back.
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
