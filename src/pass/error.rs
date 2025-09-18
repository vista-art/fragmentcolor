use thiserror::Error;

#[derive(Error, Debug)]
pub enum PassError {
    #[cfg(wasm)]
    #[error("WASM Pass Error: {0}")]
    Error(String),
}
