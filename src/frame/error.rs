use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("Missing pass in frame")]
    MissingPass,
    #[error("Duplicate edge")]
    DuplicateEdge,
    #[error("Invalid present pass")]
    InvalidPresentPass,
    #[error("Present pass must be a leaf in the DAG")]
    NotALeaf,
    #[error("Present pass must be a render pass")]
    NotRenderPass,
    #[error("Cycle detected in frame graph; falling back to insertion order")]
    CycleDetected,
    #[error("Pass '{0}' not found in frame")]
    PassNotFound(String),
    #[cfg(wasm)]
    #[error("WASM Frame Error: {0}")]
    Error(String),
}
