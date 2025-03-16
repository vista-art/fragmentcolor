use crate::error::ShaderError;
use naga::{Module, ScalarKind, Type, TypeInner, VectorSize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Debug, Clone)]
/// Represents a Uniform in the shader
pub(crate) struct Uniform {
    /// The name of the uniform
    pub(crate) name: String,
    /// The group number in the shader source
    pub(crate) group: u32,
    /// The binding number in the shader source
    pub(crate) binding: u32,
    /// The uniform data
    pub(crate) data: UniformData,
}

// @TODO consider renaming this to a more generic IO conversion type
#[cfg_attr(feature = "python", derive(FromPyObject, IntoPyObject))]
#[derive(Debug, Clone, PartialEq)]
/// Converts from User Input
pub enum UniformData {
    Bool(bool),
    UInt(u32),
    Int(i32),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    UVec2([u32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    Mat2([[f32; 2]; 2]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
    Texture(u64),
    // Array: (type, count, stride)
    Array(Vec<(UniformData, u32, u32)>),
    // Struct: name -> ((offset, name, field), struct_size)
    Struct((Vec<(u32, String, UniformData)>, u32)),
}

impl UniformData {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Bool(v) => bytemuck::bytes_of(v).to_vec(),
            Self::Float(v) => bytemuck::bytes_of(v).to_vec(),
            Self::Int(v) => bytemuck::bytes_of(v).to_vec(),
            Self::UInt(v) => bytemuck::bytes_of(v).to_vec(),
            Self::Vec2(v) => bytemuck::cast_slice(v).to_vec(),
            Self::Vec3(v) => bytemuck::cast_slice(v).to_vec(),
            Self::Vec4(v) => bytemuck::cast_slice(v).to_vec(),
            Self::IVec2(v) => bytemuck::cast_slice(v).to_vec(),
            Self::IVec3(v) => bytemuck::cast_slice(v).to_vec(),
            Self::IVec4(v) => bytemuck::cast_slice(v).to_vec(),
            Self::UVec2(v) => bytemuck::cast_slice(v).to_vec(),
            Self::UVec3(v) => bytemuck::cast_slice(v).to_vec(),
            Self::UVec4(v) => bytemuck::cast_slice(v).to_vec(),
            Self::Mat2(v) => bytemuck::cast_slice(v.as_slice()).to_vec(),
            Self::Mat3(v) => bytemuck::cast_slice(v.as_slice()).to_vec(),
            Self::Mat4(v) => bytemuck::cast_slice(v.as_slice()).to_vec(),
            Self::Texture(h) => bytemuck::bytes_of(h).to_vec(),
            Self::Array(data) => {
                let mut bytes = Vec::new();
                if let Some((data, count, _)) = data.first() {
                    for _ in 0..*count {
                        bytes.extend(data.to_bytes());
                    }
                }

                bytes
            }
            Self::Struct((fields, _)) => {
                let mut bytes = Vec::new();
                let mut previous_size = 0;
                for (offset, _, field) in fields.iter() {
                    if offset == &previous_size {
                        bytes.extend(field.to_bytes());
                    } else {
                        let padding = vec![0; (*offset - previous_size) as usize];
                        bytes.extend(padding);
                        bytes.extend(field.to_bytes());
                    }
                    previous_size = field.size();
                }
                bytes
            }
        }
    }

    pub(super) fn size(&self) -> u32 {
        match self {
            Self::Bool(_) => 1,
            Self::Float(_) => 4,
            Self::Int(_) => 4,
            Self::UInt(_) => 4,
            Self::Vec2(_) => 8,
            Self::Vec3(_) => 12,
            Self::Vec4(_) => 16,
            Self::IVec2(_) => 8,
            Self::IVec3(_) => 12,
            Self::IVec4(_) => 16,
            Self::UVec2(_) => 8,
            Self::UVec3(_) => 12,
            Self::UVec4(_) => 16,
            Self::Mat2(_) => 16,
            Self::Mat3(_) => 36,
            Self::Mat4(_) => 64,
            Self::Texture(_) => 8,
            Self::Array(data) => {
                if let Some((data, count, _)) = data.first() {
                    data.size() * count
                } else {
                    0
                }
            }
            Self::Struct((_, size)) => *size,
        }
    }
}

/// Converts from a Naga type to our internal Uniform representation
pub(crate) fn convert_type(module: &Module, ty: &Type) -> Result<UniformData, ShaderError> {
    match &ty.inner {
        TypeInner::Scalar(scalar) => Ok(match scalar.kind {
            ScalarKind::Bool => UniformData::Bool(false),
            ScalarKind::Sint => UniformData::Int(0),
            ScalarKind::Uint => UniformData::UInt(0),
            ScalarKind::Float => UniformData::Float(0.0),
            _ => return Err(ShaderError::TypeMismatch("Unsupported scalar type".into())),
        }),
        TypeInner::Vector { size, scalar, .. } if scalar.kind == ScalarKind::Float => {
            Ok(match size {
                VectorSize::Bi => UniformData::Vec2([0.0; 2]),
                VectorSize::Tri => UniformData::Vec3([0.0; 3]),
                VectorSize::Quad => UniformData::Vec4([0.0; 4]),
            })
        }
        TypeInner::Matrix { columns, rows, .. } => Ok(match (columns, rows) {
            (VectorSize::Bi, VectorSize::Bi) => UniformData::Mat2([[0.0; 2]; 2]),
            (VectorSize::Tri, VectorSize::Tri) => UniformData::Mat3([[0.0; 3]; 3]),
            (VectorSize::Quad, VectorSize::Quad) => UniformData::Mat4([[0.0; 4]; 4]),
            _ => {
                return Err(ShaderError::TypeMismatch(
                    "Unsupported matrix dimensions".into(),
                ))
            }
        }),
        TypeInner::Struct { members, span } => {
            let mut fields = Vec::new();
            for member in members {
                let name = member.name.clone().unwrap_or_default();
                let field = convert_type(module, &module.types[member.ty])?;
                fields.push((member.offset, name, field));
            }

            Ok(UniformData::Struct((fields, *span)))
        }
        TypeInner::Array { base, size, stride } => {
            let size = match size {
                naga::ArraySize::Constant(size) => size.get(),
                _ => {
                    return Err(ShaderError::TypeMismatch(
                        "Dynamic array size not supported".into(),
                    ))
                }
            };
            let base_ty = convert_type(module, &module.types[*base])?;

            let item = (base_ty, size, *stride);
            Ok(UniformData::Array(vec![item]))
        }
        _ => Err(ShaderError::TypeMismatch("Unsupported type".into())),
    }
}

// 1 element or scalar

impl From<f32> for UniformData {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<UniformData> for f32 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Float(v) => v,
            _ => 0.0,
        }
    }
}

impl From<[f32; 1]> for UniformData {
    fn from(value: [f32; 1]) -> Self {
        Self::Float(value[0])
    }
}

impl From<UniformData> for [f32; 1] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Float(v) => [v],
            _ => [0.0],
        }
    }
}

impl From<i32> for UniformData {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<UniformData> for i32 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Int(v) => v,
            _ => 0,
        }
    }
}

impl From<[i32; 1]> for UniformData {
    fn from(value: [i32; 1]) -> Self {
        Self::Int(value[0])
    }
}

impl From<UniformData> for [i32; 1] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Int(v) => [v],
            _ => [0],
        }
    }
}

impl From<u32> for UniformData {
    fn from(value: u32) -> Self {
        Self::UInt(value)
    }
}

impl From<UniformData> for u32 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::UInt(v) => v,
            _ => 0,
        }
    }
}

impl From<[u32; 1]> for UniformData {
    fn from(value: [u32; 1]) -> Self {
        Self::UInt(value[0])
    }
}

impl From<UniformData> for [u32; 1] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::UInt(v) => [v],
            _ => [0],
        }
    }
}

// 2 elements

impl From<[f32; 2]> for UniformData {
    fn from(value: [f32; 2]) -> Self {
        Self::Vec2(value)
    }
}

impl From<UniformData> for [f32; 2] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec2(v) => v,
            _ => [0.0; 2],
        }
    }
}

impl From<(f32, f32)> for UniformData {
    fn from(value: (f32, f32)) -> Self {
        Self::Vec2([value.0, value.1])
    }
}

impl From<UniformData> for (f32, f32) {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec2(v) => (v[0], v[1]),
            _ => (0.0, 0.0),
        }
    }
}

impl From<glam::Vec2> for UniformData {
    fn from(v: glam::Vec2) -> Self {
        Self::Vec2(v.to_array())
    }
}

impl From<UniformData> for glam::Vec2 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec2(v) => glam::Vec2::from(v),
            _ => glam::Vec2::ZERO,
        }
    }
}

// 3 elements

impl From<[f32; 3]> for UniformData {
    fn from(value: [f32; 3]) -> Self {
        Self::Vec3(value)
    }
}

impl From<UniformData> for [f32; 3] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec3(v) => v,
            _ => [0.0; 3],
        }
    }
}

impl From<(f32, f32, f32)> for UniformData {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::Vec3([value.0, value.1, value.2])
    }
}

impl From<UniformData> for (f32, f32, f32) {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec3(v) => (v[0], v[1], v[2]),
            _ => (0.0, 0.0, 0.0),
        }
    }
}

impl From<glam::Vec3> for UniformData {
    fn from(v: glam::Vec3) -> Self {
        Self::Vec3(v.to_array())
    }
}

impl From<UniformData> for glam::Vec3 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec3(v) => glam::Vec3::from(v),
            _ => glam::Vec3::ZERO,
        }
    }
}

// 4 elements

impl From<[f32; 4]> for UniformData {
    fn from(value: [f32; 4]) -> Self {
        Self::Vec4(value)
    }
}

impl From<UniformData> for [f32; 4] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec4(v) => v,
            _ => [0.0; 4],
        }
    }
}

impl From<(f32, f32, f32, f32)> for UniformData {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::Vec4([value.0, value.1, value.2, value.3])
    }
}

impl From<UniformData> for (f32, f32, f32, f32) {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec4(v) => (v[0], v[1], v[2], v[3]),
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl From<glam::Vec4> for UniformData {
    fn from(v: glam::Vec4) -> Self {
        Self::Vec4(v.to_array())
    }
}

impl From<UniformData> for glam::Vec4 {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec4(v) => glam::Vec4::from(v),
            _ => glam::Vec4::ZERO,
        }
    }
}

impl From<wgpu::Extent3d> for UniformData {
    fn from(value: wgpu::Extent3d) -> Self {
        Self::UVec3([value.width, value.height, value.depth_or_array_layers])
    }
}

impl From<UniformData> for wgpu::Extent3d {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::Vec2([w, h]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: 1,
            },
            UniformData::Vec3([w, h, d]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: d as u32,
            },
            UniformData::Vec4([w, h, d, _]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: d as u32,
            },
            UniformData::UVec2([w, h]) => wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            UniformData::UVec3([w, h, d]) => wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: d,
            },
            UniformData::UVec4([w, h, d, _]) => wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: d,
            },
            UniformData::IVec2([w, h]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: 1,
            },
            UniformData::IVec3([w, h, d]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: d as u32,
            },
            UniformData::IVec4([w, h, d, _]) => wgpu::Extent3d {
                width: w as u32,
                height: h as u32,
                depth_or_array_layers: d as u32,
            },
            _ => wgpu::Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 0,
            },
        }
    }
}
