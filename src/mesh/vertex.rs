use super::builtins::VertexPosition;
use lsp_doc::lsp_doc;
use std::collections::HashMap;

#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/vertex/vertex.md")]
pub struct Vertex {
    pub(crate) position: VertexPosition,
    pub(crate) dimensions: u8,
    pub(crate) properties: HashMap<String, VertexValue>,
    pub(crate) prop_locations: HashMap<String, u32>,
    pub(crate) next_location: u32,
}

pub trait IntoVertexPositionFull {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8);
}

impl IntoVertexPositionFull for [f32; 2] {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self[0], self[1], 0.0, 1.0), 2)
    }
}
impl IntoVertexPositionFull for [f32; 3] {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self[0], self[1], self[2], 1.0), 3)
    }
}
impl IntoVertexPositionFull for [f32; 4] {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::from(self), 4)
    }
}
impl IntoVertexPositionFull for (f32, f32) {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self.0, self.1, 0.0, 1.0), 2)
    }
}
impl IntoVertexPositionFull for (f32, f32, f32) {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self.0, self.1, self.2, 1.0), 3)
    }
}
impl IntoVertexPositionFull for (f32, f32, f32, f32) {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self.0, self.1, self.2, self.3), 4)
    }
}
impl IntoVertexPositionFull for f32 {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self, 0.0, 0.0, 1.0), 1)
    }
}
impl IntoVertexPositionFull for (u32, u32) {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self.0 as f32, self.1 as f32, 0.0, 1.0), 2)
    }
}
impl IntoVertexPositionFull for (u32, u32, u32) {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (
            glam::Vec4::new(self.0 as f32, self.1 as f32, self.2 as f32, 1.0),
            3,
        )
    }
}
impl IntoVertexPositionFull for [u32; 2] {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (glam::Vec4::new(self[0] as f32, self[1] as f32, 0.0, 1.0), 2)
    }
}
impl IntoVertexPositionFull for [u32; 3] {
    fn into_v4_and_dimensions(self) -> (glam::Vec4, u8) {
        (
            glam::Vec4::new(self[0] as f32, self[1] as f32, self[2] as f32, 1.0),
            3,
        )
    }
}

impl Vertex {
    #[lsp_doc("docs/api/core/vertex/new.md")]
    pub fn new<P: IntoVertexPositionFull>(position: P) -> Self {
        let (v4, dimensions) = position.into_v4_and_dimensions();
        Self {
            position: VertexPosition(v4),
            dimensions,
            properties: HashMap::new(),
            prop_locations: HashMap::new(),
            next_location: 1,
        }
    }

    #[lsp_doc("docs/api/core/vertex/with.md")]
    pub fn with<V: Into<VertexValue>>(mut self, key: &str, v: V) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, v.into());
        self
    }

    #[lsp_doc("docs/api/core/vertex/create_instance.md")]
    pub fn create_instance(&self) -> Instance {
        // Instances do not implicitly copy position; only explicit per-instance properties are carried.
        Instance {
            properties: self.properties.clone(),
            prop_locations: self.prop_locations.clone(),
        }
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
        Vertex::new(p).with("uv", uv)
    }
}
impl From<([f32; 3], [f32; 2], [f32; 4])> for Vertex {
    fn from((p, uv, c): ([f32; 3], [f32; 2], [f32; 4])) -> Self {
        Vertex::new(p).with("uv", uv).with("color", c)
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
    pub(crate) properties: HashMap<String, VertexValue>,
    pub(crate) prop_locations: HashMap<String, u32>,
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

// Convenient conversions so callers can use plain arrays/scalars
impl From<f32> for VertexValue {
    fn from(v: f32) -> Self {
        VertexValue::F32(v)
    }
}
impl From<[f32; 2]> for VertexValue {
    fn from(v: [f32; 2]) -> Self {
        VertexValue::F32x2(v)
    }
}
impl From<[f32; 3]> for VertexValue {
    fn from(v: [f32; 3]) -> Self {
        VertexValue::F32x3(v)
    }
}
impl From<[f32; 4]> for VertexValue {
    fn from(v: [f32; 4]) -> Self {
        VertexValue::F32x4(v)
    }
}
impl From<u32> for VertexValue {
    fn from(v: u32) -> Self {
        VertexValue::U32(v)
    }
}
impl From<[u32; 2]> for VertexValue {
    fn from(v: [u32; 2]) -> Self {
        VertexValue::U32x2(v)
    }
}
impl From<[u32; 3]> for VertexValue {
    fn from(v: [u32; 3]) -> Self {
        VertexValue::U32x3(v)
    }
}
impl From<[u32; 4]> for VertexValue {
    fn from(v: [u32; 4]) -> Self {
        VertexValue::U32x4(v)
    }
}
impl From<i32> for VertexValue {
    fn from(v: i32) -> Self {
        VertexValue::I32(v)
    }
}
impl From<[i32; 2]> for VertexValue {
    fn from(v: [i32; 2]) -> Self {
        VertexValue::I32x2(v)
    }
}
impl From<[i32; 3]> for VertexValue {
    fn from(v: [i32; 3]) -> Self {
        VertexValue::I32x3(v)
    }
}
impl From<[i32; 4]> for VertexValue {
    fn from(v: [i32; 4]) -> Self {
        VertexValue::I32x4(v)
    }
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
