use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    /// A region with no pixels, which no viewport or scissor can use.
    #[error("a region needs a width and a height of at least 1, got {width} by {height}")]
    Empty { width: u32, height: u32 },
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
