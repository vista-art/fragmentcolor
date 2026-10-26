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
    /// An index from `set_indices` past the last vertex the mesh holds.
    #[error("Index {index} points past the last of {vertices} vertices")]
    IndexRange { index: u32, vertices: usize },
    /// Mesh data that needs a GPU buffer larger than the device makes.
    #[error("The {buffer} buffer takes {size} bytes, past this device's max_buffer_size ({limit})")]
    BufferSize {
        buffer: &'static str,
        size: u64,
        limit: u64,
    },
    #[cfg(wasm)]
    #[error("WASM Mesh Error: {0}")]
    Error(String),
}
