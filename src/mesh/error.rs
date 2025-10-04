#[derive(thiserror::Error, Debug)]
pub enum MeshError {
    #[error("Invalid vertex: {0}")]
    InvalidVertex(String),
    #[error("Invalid instance: {0}")]
    InvalidInstance(String),
    #[error("Mismatched vertex layout: {0}")]
    LayoutMismatch(String),
    #[error("Missing vertex schema")]
    NoVertexSchema,
    #[error("Missing instance schema")]
    NoInstanceSchema,
    #[error("GPU streams not initialized")]
    NoGpuStreams,
    #[cfg(wasm)]
    #[error("WASM Mesh Error: {0}")]
    Error(String),
}
