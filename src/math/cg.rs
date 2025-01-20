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
