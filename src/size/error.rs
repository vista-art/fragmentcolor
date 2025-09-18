use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizeError {
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    #[cfg(wasm)]
    #[error("WASM Size Error: {0}")]
    Error(String),
}
