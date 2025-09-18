use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameError {
    #[cfg(wasm)]
    #[error("WASM Frame Error: {0}")]
    Error(String),
}
