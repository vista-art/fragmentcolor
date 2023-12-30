pub type Pixel = mint::Point2<u16>;

pub type Point2 = mint::Point2<f32>;
pub type Point3 = mint::Point3<f32>;

pub type Mat2 = mint::ColumnMatrix2<f32>;
pub type Mat3 = mint::ColumnMatrix3<f32>;
pub type Mat4 = mint::ColumnMatrix4<f32>;

pub type Matrix2 = mint::ColumnMatrix2<f32>;
pub type Matrix3 = mint::ColumnMatrix3<f32>;
pub type Matrix4 = mint::ColumnMatrix4<f32>;

pub type Mat2x3 = mint::ColumnMatrix2x3<f32>;
pub type Mat2x4 = mint::ColumnMatrix2x4<f32>;
pub type Mat3x2 = mint::ColumnMatrix3x2<f32>;
pub type Mat3x4 = mint::ColumnMatrix3x4<f32>;
pub type Mat4x2 = mint::ColumnMatrix4x2<f32>;
pub type Mat4x3 = mint::ColumnMatrix4x3<f32>;

pub type Matrix2x3 = mint::ColumnMatrix2x3<f32>;
pub type Matrix2x4 = mint::ColumnMatrix2x4<f32>;
pub type Matrix3x2 = mint::ColumnMatrix3x2<f32>;
pub type Matrix3x4 = mint::ColumnMatrix3x4<f32>;
pub type Matrix4x2 = mint::ColumnMatrix4x2<f32>;
pub type Matrix4x3 = mint::ColumnMatrix4x3<f32>;

pub type Vec2 = mint::Vector2<f32>;
pub type Vec3 = mint::Vector3<f32>;
pub type Vec4 = mint::Vector4<f32>;

pub type Vector2 = mint::Vector2<f32>;
pub type Vector3 = mint::Vector3<f32>;
pub type Vector4 = mint::Vector4<f32>;

pub type EulerAngles = mint::EulerAngles<f32, f32>;
pub type Quaternion = mint::Quaternion<f32>;

pub const ORIGIN: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub const UP_VECTOR: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};

pub enum Vec2or3 {
    Vec2(Vec2),
    Vec3(Vec3),
}

impl From<Vec2> for Vec2or3 {
    fn from(v: Vec2) -> Self {
        Self::Vec2(v)
    }
}

impl From<Vec3> for Vec2or3 {
    fn from(v: Vec3) -> Self {
        Self::Vec3(v)
    }
}

impl From<Vec2or3> for Vec2 {
    fn from(v: Vec2or3) -> Self {
        match v {
            Vec2or3::Vec2(v) => v,
            Vec2or3::Vec3(v) => Vec2 { x: v.x, y: v.y },
        }
    }
}

impl From<Vec2or3> for Vec3 {
    fn from(v: Vec2or3) -> Self {
        match v {
            Vec2or3::Vec2(v) => Vec3 {
                x: v.x,
                y: v.y,
                z: 0.0,
            },
            Vec2or3::Vec3(v) => v,
        }
    }
}

impl From<Vec2or3> for Point2 {
    fn from(v: Vec2or3) -> Self {
        match v {
            Vec2or3::Vec2(v) => Point2 { x: v.x, y: v.y },
            Vec2or3::Vec3(v) => Point2 { x: v.x, y: v.y },
        }
    }
}

impl From<[f32; 3]> for Vec2or3 {
    fn from(v: [f32; 3]) -> Self {
        Vec2or3::Vec3(v.into())
    }
}

impl From<[f32; 2]> for Vec2or3 {
    fn from(v: [f32; 2]) -> Self {
        Vec2or3::Vec2(v.into())
    }
}

impl From<(f32, f32)> for Vec2or3 {
    fn from(v: (f32, f32)) -> Self {
        Vec2or3::Vec2([v.0, v.1].into())
    }
}

impl From<(f32, f32, f32)> for Vec2or3 {
    fn from(v: (f32, f32, f32)) -> Self {
        Vec2or3::Vec3([v.0, v.1, v.2].into())
    }
}
