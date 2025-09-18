use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM Color Error: {0}")]
    Error(String),
}
