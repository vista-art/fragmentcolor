use glam::Vec4;

/// Vertex position stored as a Vec4, mirroring GPU conventions (xyzw),
/// with w defaulting to 1.0 when not provided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VertexPosition(pub(crate) Vec4);

/// Vertex index stored as a u32
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VertexIndex(pub(crate) u32);

/// Instance index stored as a u32
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InstanceIndex(pub(crate) u32);

impl From<f32> for VertexPosition {
    fn from(x: f32) -> Self {
        VertexPosition(Vec4::new(x, 0.0, 0.0, 1.0))
    }
}
impl From<(f32, f32)> for VertexPosition {
    fn from(p: (f32, f32)) -> Self {
        VertexPosition(Vec4::new(p.0, p.1, 0.0, 1.0))
    }
}
impl From<(f32, f32, f32)> for VertexPosition {
    fn from(p: (f32, f32, f32)) -> Self {
        VertexPosition(Vec4::new(p.0, p.1, p.2, 1.0))
    }
}
impl From<(f32, f32, f32, f32)> for VertexPosition {
    fn from(p: (f32, f32, f32, f32)) -> Self {
        VertexPosition(Vec4::new(p.0, p.1, p.2, p.3))
    }
}

impl From<[f32; 2]> for VertexPosition {
    fn from(a: [f32; 2]) -> Self {
        VertexPosition(Vec4::new(a[0], a[1], 0.0, 1.0))
    }
}
impl From<[f32; 3]> for VertexPosition {
    fn from(a: [f32; 3]) -> Self {
        VertexPosition(Vec4::new(a[0], a[1], a[2], 1.0))
    }
}
impl From<[f32; 4]> for VertexPosition {
    fn from(a: [f32; 4]) -> Self {
        VertexPosition(Vec4::new(a[0], a[1], a[2], a[3]))
    }
}

// Integer variants are accepted and cast to f32 for convenience.
impl From<(u32, u32)> for VertexPosition {
    fn from(p: (u32, u32)) -> Self {
        VertexPosition::from((p.0 as f32, p.1 as f32))
    }
}
impl From<(u32, u32, u32)> for VertexPosition {
    fn from(p: (u32, u32, u32)) -> Self {
        VertexPosition::from((p.0 as f32, p.1 as f32, p.2 as f32))
    }
}
impl From<[u32; 2]> for VertexPosition {
    fn from(a: [u32; 2]) -> Self {
        VertexPosition::from([a[0] as f32, a[1] as f32])
    }
}
impl From<[u32; 3]> for VertexPosition {
    fn from(a: [u32; 3]) -> Self {
        VertexPosition::from([a[0] as f32, a[1] as f32, a[2] as f32])
    }
}
