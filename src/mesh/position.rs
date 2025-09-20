use glam::Vec4;

/// Vertex position stored as a Vec4, mirroring GPU conventions (xyzw),
/// with w defaulting to 1.0 when not provided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexPosition {
    pub v: Vec4,
    /// Number of provided spatial components (1, 2, 3, or 4).
    /// This is used by current mesh packing to decide the vertex format (e.g., Float32x2 or Float32x3).
    pub comps: u8,
}

impl From<f32> for VertexPosition {
    fn from(x: f32) -> Self {
        VertexPosition { v: Vec4::new(x, 0.0, 0.0, 1.0), comps: 1 }
    }
}
impl From<(f32, f32)> for VertexPosition {
    fn from(p: (f32, f32)) -> Self {
        VertexPosition { v: Vec4::new(p.0, p.1, 0.0, 1.0), comps: 2 }
    }
}
impl From<(f32, f32, f32)> for VertexPosition {
    fn from(p: (f32, f32, f32)) -> Self {
        VertexPosition { v: Vec4::new(p.0, p.1, p.2, 1.0), comps: 3 }
    }
}
impl From<(f32, f32, f32, f32)> for VertexPosition {
    fn from(p: (f32, f32, f32, f32)) -> Self {
        VertexPosition { v: Vec4::new(p.0, p.1, p.2, p.3), comps: 4 }
    }
}

impl From<[f32; 2]> for VertexPosition {
    fn from(a: [f32; 2]) -> Self {
        VertexPosition { v: Vec4::new(a[0], a[1], 0.0, 1.0), comps: 2 }
    }
}
impl From<[f32; 3]> for VertexPosition {
    fn from(a: [f32; 3]) -> Self {
        VertexPosition { v: Vec4::new(a[0], a[1], a[2], 1.0), comps: 3 }
    }
}
impl From<[f32; 4]> for VertexPosition {
    fn from(a: [f32; 4]) -> Self {
        VertexPosition { v: Vec4::new(a[0], a[1], a[2], a[3]), comps: 4 }
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
