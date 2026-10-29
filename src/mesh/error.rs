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
    #[error("A vertex attribute takes 1 to 4 numbers, got {0}")]
    Length(usize),
    #[error("A mesh holds more indices or instances than a 32-bit count reaches")]
    Count(#[from] std::num::TryFromIntError),
    /// An index list from `set_indices` that ends partway through a triangle.
    #[error("A triangle list takes a multiple of 3 indices, got {0}")]
    IndexCount(usize),
    /// A mesh without `set_indices` whose vertices, drawn in threes, end partway through a triangle.
    #[error("A mesh without set_indices draws its vertices in threes, and it holds {0}")]
    VertexCount(usize),
    /// An index from `set_indices` past the last vertex the mesh holds.
    #[error("Index {index} points past the last of {vertices} vertices")]
    IndexRange { index: u32, vertices: usize },
    #[cfg(wasm)]
    #[error("WASM Mesh Error: {0}")]
    Error(String),
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for MeshError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::Base
    }
}
