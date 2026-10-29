use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
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
    /// The size is the one asked for, which two corners can put at `u32::MAX + 1`.
    #[error(
        "a region at {x}, {y} sized {width} by {height} reaches past {}",
        u32::MAX
    )]
    Overflow {
        x: u32,
        y: u32,
        width: u64,
        height: u64,
    },
    #[cfg(wasm)]
    #[error("WASM ScreenRegion Error: {0}")]
    Error(String),
}

#[cfg(python)]
impl From<ScreenRegionError> for pyo3::PyErr {
    fn from(error: ScreenRegionError) -> Self {
        crate::error::InvalidInputError::new_err(error.to_string())
    }
}
