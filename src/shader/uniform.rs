use crate::error::ShaderError;
use naga::{Module, ScalarKind, Type, TypeInner, VectorSize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Uniform {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) layout: UniformLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformLayout {
    pub size: u32,
    pub ty: UniformType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UniformType {
    // Scalars
    Bool,
    Float,
    Int,
    UInt,
    // Vectors
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    // Matrices
    Mat2,
    Mat3,
    Mat4,
    // Texture Handle
    Texture,
    // Array (type, size, stride)
    Array(Box<UniformType>, u32, u32),
    // Struct (offset, type)
    Struct(HashMap<String, (u32, UniformType)>),
}

impl UniformType {
    pub fn size(&self) -> u32 {
        match self {
            UniformType::Bool => 1,
            UniformType::Int => 4,
            UniformType::UInt => 4,
            UniformType::Float => 4,
            UniformType::IVec2 => 8,
            UniformType::IVec3 => 12,
            UniformType::IVec4 => 16,
            UniformType::UVec2 => 8,
            UniformType::UVec3 => 12,
            UniformType::UVec4 => 16,
            UniformType::Vec2 => 8,
            UniformType::Vec3 => 12,
            UniformType::Vec4 => 16,
            UniformType::Mat2 => 16,
            UniformType::Mat3 => 36,
            UniformType::Mat4 => 64,
            UniformType::Texture => 8,
            UniformType::Array(ty, length, _) => ty.size() * length,
            UniformType::Struct(fields) => fields.values().map(|(_, ty)| ty.size()).sum(),
        }
    }
}

impl UniformLayout {
    pub(crate) fn from_naga_type(module: &Module, ty: &Type) -> Result<Self, ShaderError> {
        let size = ty.inner.size(module.to_ctx());
        let uniform_type = convert_type(module, ty)?;
        Ok(Self {
            size,
            ty: uniform_type,
        })
    }
}

/// Converts from a Naga type to our internal Uniform representation
fn convert_type(module: &Module, ty: &Type) -> Result<UniformType, ShaderError> {
    match &ty.inner {
        TypeInner::Scalar(scalar) => Ok(match scalar.kind {
            ScalarKind::Bool => UniformType::Bool,
            ScalarKind::Sint => UniformType::Int,
            ScalarKind::Uint => UniformType::UInt,
            ScalarKind::Float => UniformType::Float,
            _ => return Err(ShaderError::TypeMismatch("Unsupported scalar type".into())),
        }),
        TypeInner::Vector { size, scalar, .. } if scalar.kind == ScalarKind::Float => {
            Ok(match size {
                VectorSize::Bi => UniformType::Vec2,
                VectorSize::Tri => UniformType::Vec3,
                VectorSize::Quad => UniformType::Vec4,
            })
        }
        TypeInner::Matrix { columns, rows, .. } => Ok(match (columns, rows) {
            (VectorSize::Bi, VectorSize::Bi) => UniformType::Mat2,
            (VectorSize::Tri, VectorSize::Tri) => UniformType::Mat3,
            (VectorSize::Quad, VectorSize::Quad) => UniformType::Mat4,
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

            Ok(UniformType::Struct(fields))
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

            Ok(UniformType::Array(Box::new(base_ty), size, *stride))
        }
        _ => Err(ShaderError::TypeMismatch("Unsupported type".into())),
    }
}
