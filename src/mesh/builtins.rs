use glam::Vec4;

/// Vertex position stored as a Vec4, mirroring GPU conventions (xyzw),
/// with w defaulting to 1.0 when not provided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VertexPosition(pub(crate) Vec4);

/// Vertex index stored as a u32
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct _VertexIndex(pub(crate) u32);

/// Instance index stored as a u32
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct _InstanceIndex(pub(crate) u32);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4::new(x, y, z, w)
    }

    #[test]
    fn from_scalar_and_tuples() {
        let a = VertexPosition::from(2.0f32);
        assert_eq!(a.0, v4(2.0, 0.0, 0.0, 1.0));

        let b = VertexPosition::from((1.0f32, 2.0f32));
        assert_eq!(b.0, v4(1.0, 2.0, 0.0, 1.0));

        let c = VertexPosition::from((3.0f32, 4.0f32, 5.0f32));
        assert_eq!(c.0, v4(3.0, 4.0, 5.0, 1.0));

        let d = VertexPosition::from((6.0f32, 7.0f32, 8.0f32, 9.0f32));
        assert_eq!(d.0, v4(6.0, 7.0, 8.0, 9.0));
    }

    #[test]
    fn from_float_arrays() {
        let a2 = VertexPosition::from([1.0f32, 2.0]);
        assert_eq!(a2.0, v4(1.0, 2.0, 0.0, 1.0));

        let a3 = VertexPosition::from([3.0f32, 4.0, 5.0]);
        assert_eq!(a3.0, v4(3.0, 4.0, 5.0, 1.0));

        let a4 = VertexPosition::from([6.0f32, 7.0, 8.0, 9.0]);
        assert_eq!(a4.0, v4(6.0, 7.0, 8.0, 9.0));
    }

    #[test]
    fn from_unsigned_ints() {
        let t2 = VertexPosition::from((1u32, 2u32));
        assert_eq!(t2.0, v4(1.0, 2.0, 0.0, 1.0));

        let t3 = VertexPosition::from((3u32, 4u32, 5u32));
        assert_eq!(t3.0, v4(3.0, 4.0, 5.0, 1.0));

        let a2 = VertexPosition::from([6u32, 7u32]);
        assert_eq!(a2.0, v4(6.0, 7.0, 0.0, 1.0));

        let a3 = VertexPosition::from([8u32, 9u32, 10u32]);
        assert_eq!(a3.0, v4(8.0, 9.0, 10.0, 1.0));
    }
}
