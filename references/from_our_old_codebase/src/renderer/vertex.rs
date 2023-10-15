pub trait Vertex {
    fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a>;
}
