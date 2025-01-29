use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("Failed to parse shader: {0}")]
    ParseError(String),
    #[error("Uniform not found: {0}")]
    UniformNotFound(String),
    #[error("Type mismatch for uniform {0}")]
    TypeMismatch(String),
    #[error("Field not found in struct: {0}")]
    FieldNotFound(String),
    #[error("WGSL error: {0}")]
    WgslError(#[from] naga::front::wgsl::ParseError),
    #[error("WGPU error: {0}")]
    WgpuError(#[from] wgpu::Error),
}
