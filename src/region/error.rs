use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenRegionError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM ScreenRegion Error: {0}")]
    Error(String),
}
