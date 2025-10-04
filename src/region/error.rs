use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegionError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM Region Error: {0}")]
    Error(String),
}
