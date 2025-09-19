#[derive(thiserror::Error, Debug)]
pub enum MeshError {
    #[error("Invalid vertex: {0}")]
    InvalidVertex(String),
    #[error("Invalid instance: {0}")]
    InvalidInstance(String),
    #[error("Mismatched vertex layout: {0}")]
    LayoutMismatch(String),
}
