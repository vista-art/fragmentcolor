use crate::error::ShaderError;
use naga::{Module, ScalarKind, Type, TypeInner, VectorSize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
///
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

#[derive(Debug, Clone, PartialEq)]
/// Converts from User Input
pub enum UniformData {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    UVec2([u32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    Mat2([[f32; 2]; 2]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
    Texture(u64),
    // Array: (type, count, stride)
    Array(Box<UniformData>, u32, u32),
    // Struct: name -> (offset, data)
    Struct(HashMap<String, (u32, UniformData)>),
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
            Self::Array(data, count, _) => {
                let mut bytes = Vec::new();
                for _ in 0..*count {
                    bytes.extend(data.to_bytes());
                }
                bytes
            }
            Self::Struct(s) => {
                let mut bytes = Vec::new();
                for (_, (_, data)) in s {
                    bytes.extend(data.to_bytes());
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
            Self::Array(v, count, _) => v.size() * count,
            Self::Struct(s) => s.values().map(|(_, data)| data.size()).sum(),
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
        TypeInner::Struct { members, .. } => {
            let mut fields = HashMap::new();
            for member in members {
                let name = member.name.clone().unwrap_or_default();
                let member_ty = convert_type(module, &module.types[member.ty])?;
                fields.insert(name, (member.offset, member_ty));
            }

            Ok(UniformData::Struct(fields))
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

            Ok(UniformData::Array(Box::new(base_ty), size, *stride))
        }
        _ => Err(ShaderError::TypeMismatch("Unsupported type".into())),
    }
}

impl From<f32> for UniformData {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<[f32; 1]> for UniformData {
    fn from(value: [f32; 1]) -> Self {
        Self::Float(value[0])
    }
}

impl From<i32> for UniformData {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<[i32; 1]> for UniformData {
    fn from(value: [i32; 1]) -> Self {
        Self::Int(value[0])
    }
}

impl From<u32> for UniformData {
    fn from(value: u32) -> Self {
        Self::UInt(value)
    }
}

impl From<[u32; 1]> for UniformData {
    fn from(value: [u32; 1]) -> Self {
        Self::UInt(value[0])
    }
}

impl From<[f32; 2]> for UniformData {
    fn from(value: [f32; 2]) -> Self {
        Self::Vec2(value)
    }
}

impl From<(f32, f32)> for UniformData {
    fn from(value: (f32, f32)) -> Self {
        Self::Vec2([value.0, value.1])
    }
}

impl From<glam::Vec2> for UniformData {
    fn from(v: glam::Vec2) -> Self {
        Self::Vec2(v.to_array())
    }
}

impl From<[f32; 3]> for UniformData {
    fn from(value: [f32; 3]) -> Self {
        Self::Vec3(value)
    }
}

impl From<(f32, f32, f32)> for UniformData {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::Vec3([value.0, value.1, value.2])
    }
}

impl From<glam::Vec3> for UniformData {
    fn from(v: glam::Vec3) -> Self {
        Self::Vec3(v.to_array())
    }
}

impl From<[f32; 4]> for UniformData {
    fn from(value: [f32; 4]) -> Self {
        Self::Vec4(value)
    }
}

impl From<(f32, f32, f32, f32)> for UniformData {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::Vec4([value.0, value.1, value.2, value.3])
    }
}

impl From<glam::Vec4> for UniformData {
    fn from(v: glam::Vec4) -> Self {
        Self::Vec4(v.to_array())
    }
}
