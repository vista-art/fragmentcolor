use crate::Position;
use lsp_doc::lsp_doc;
use std::collections::HashMap;

#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/vertex/vertex.md")]
pub struct Vertex {
    pub(crate) position: Position,
    pub(crate) properties: HashMap<String, VertexValue>,
}

impl Vertex {
    #[lsp_doc("docs/api/core/vertex/new.md")]
    pub fn new(position: impl Into<Position>) -> Self {
        Self {
            position: position.into(),
            properties: HashMap::new(),
        }
    }
    #[lsp_doc("docs/api/core/vertex/with_uv.md")]
    pub fn with_uv(mut self, uv: [f32; 2]) -> Self {
        self.properties.insert("uv".into(), VertexValue::F32x2(uv));
        self
    }
    #[lsp_doc("docs/api/core/vertex/with_color.md")]
    pub fn with_color(mut self, rgba: [f32; 4]) -> Self {
        self.properties
            .insert("color".into(), VertexValue::F32x4(rgba));
        self
    }
    #[lsp_doc("docs/api/core/vertex/with_property.md")]
    pub fn with_property(mut self, key: &str, v: VertexValue) -> Self {
        self.properties.insert(key.into(), v);
        self
    }
    #[lsp_doc("docs/api/core/vertex/create_instance.md")]
    pub fn create_instance(&self) -> Instance {
        let mut props = self.properties.clone();
        // Treat position as a regular prop for instances (if shader wants it)
        match self.position {
            Position::Pos2(v) => {
                props.insert("position2".into(), VertexValue::F32x2(v));
            }
            Position::Pos3(v) => {
                props.insert("position3".into(), VertexValue::F32x3(v));
            }
        }
        Instance { props }
    }
}

impl From<[f32; 2]> for Vertex {
    fn from(v: [f32; 2]) -> Self {
        Vertex::new(v)
    }
}
impl From<[f32; 3]> for Vertex {
    fn from(v: [f32; 3]) -> Self {
        Vertex::new(v)
    }
}
impl From<([f32; 3], [f32; 2])> for Vertex {
    fn from((p, uv): ([f32; 3], [f32; 2])) -> Self {
        Vertex::new(p).with_uv(uv)
    }
}
impl From<([f32; 3], [f32; 2], [f32; 4])> for Vertex {
    fn from((p, uv, c): ([f32; 3], [f32; 2], [f32; 4])) -> Self {
        Vertex::new(Position::Pos3(p)).with_uv(uv).with_color(c)
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.properties == other.properties
    }
}
impl Eq for Vertex {}

#[derive(Clone, Debug, Default)]
pub struct Instance {
    pub(crate) props: HashMap<String, VertexValue>,
}

impl From<Vertex> for Instance {
    fn from(v: Vertex) -> Self {
        v.create_instance()
    }
}
impl From<&Vertex> for Instance {
    fn from(v: &Vertex) -> Self {
        v.create_instance()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VertexValue {
    F32(f32),
    F32x2([f32; 2]),
    F32x3([f32; 3]),
    F32x4([f32; 4]),
    U32(u32),
    U32x2([u32; 2]),
    U32x3([u32; 3]),
    U32x4([u32; 4]),
    I32(i32),
    I32x2([i32; 2]),
    I32x3([i32; 3]),
    I32x4([i32; 4]),
}

impl VertexValue {
    pub(crate) fn size(&self) -> u64 {
        match self {
            VertexValue::F32(_) | VertexValue::U32(_) | VertexValue::I32(_) => 4,
            VertexValue::F32x2(_) | VertexValue::U32x2(_) | VertexValue::I32x2(_) => 8,
            VertexValue::F32x3(_) | VertexValue::U32x3(_) | VertexValue::I32x3(_) => 12,
            VertexValue::F32x4(_) | VertexValue::U32x4(_) | VertexValue::I32x4(_) => 16,
        }
    }
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        match self {
            VertexValue::F32(v) => bytemuck::bytes_of(v).to_vec(),
            VertexValue::F32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::F32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::F32x4(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32(v) => bytemuck::bytes_of(v).to_vec(),
            VertexValue::U32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32x4(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32(v) => bytemuck::cast_slice(&[*v as u32]).to_vec(),
            VertexValue::I32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32x4(v) => bytemuck::cast_slice(v).to_vec(),
        }
    }
    pub(crate) fn format(&self) -> wgpu::VertexFormat {
        match self {
            VertexValue::F32(_) => wgpu::VertexFormat::Float32,
            VertexValue::F32x2(_) => wgpu::VertexFormat::Float32x2,
            VertexValue::F32x3(_) => wgpu::VertexFormat::Float32x3,
            VertexValue::F32x4(_) => wgpu::VertexFormat::Float32x4,
            VertexValue::U32(_) => wgpu::VertexFormat::Uint32,
            VertexValue::U32x2(_) => wgpu::VertexFormat::Uint32x2,
            VertexValue::U32x3(_) => wgpu::VertexFormat::Uint32x3,
            VertexValue::U32x4(_) => wgpu::VertexFormat::Uint32x4,
            VertexValue::I32(_) => wgpu::VertexFormat::Sint32,
            VertexValue::I32x2(_) => wgpu::VertexFormat::Sint32x2,
            VertexValue::I32x3(_) => wgpu::VertexFormat::Sint32x3,
            VertexValue::I32x4(_) => wgpu::VertexFormat::Sint32x4,
        }
    }
}
