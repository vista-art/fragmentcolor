//! Re-export of mint math types.
//!
//! We abbreviate the most commonly used Linear Algebra types,
//! Namely Vector (Vec) and Matix (Mat).
//!
//! All matrices in PLRender are Column Major.

pub use mint::Point2;
pub use mint::Point3;

pub use mint::ColumnMatrix2 as Mat2;
pub use mint::ColumnMatrix3 as Mat3;
pub use mint::ColumnMatrix4 as Mat4;

pub use mint::ColumnMatrix2x3 as Matrix2x3;
pub use mint::ColumnMatrix2x4 as Matrix2x4;
pub use mint::ColumnMatrix3x2 as Matrix3x2;
pub use mint::ColumnMatrix3x4 as Matrix3x4;
pub use mint::ColumnMatrix4x2 as Matrix4x2;
pub use mint::ColumnMatrix4x3 as Matrix4x3;

pub use mint::Vector2 as Vec2;
pub use mint::Vector3 as Vec3;
pub use mint::Vector4 as Vec4;

pub use mint::EulerAngles;
pub use mint::Quaternion;

pub const ORIGIN: mint::Vector3<f32> = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub const UP_VECTOR: mint::Vector3<f32> = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};
