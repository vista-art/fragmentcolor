use crate::error::ShaderError;
use naga::{Module, ScalarKind, Type, TypeInner, VectorSize};
use std::collections::HashMap;

pub(crate) struct Uniform {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) layout: UniformLayout,
    pub(crate) cpu_buffer: Vec<u8>,
    pub(crate) gpu_buffer: wgpu::Buffer,
    pub(crate) dirty: bool,
}

#[derive(Debug, Clone)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Struct(HashMap<String, (u32, UniformType)>),
}

#[derive(Debug, Clone)]
pub(crate) struct UniformLayout {
    pub(crate) size: u32,
    pub(crate) ty: UniformType,
}

#[derive(Debug, Clone)]
pub(crate) struct UniformMetadata {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) layout: UniformLayout,
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

fn convert_type(module: &Module, ty: &Type) -> Result<UniformType, ShaderError> {
    match &ty.inner {
        TypeInner::Scalar(scalar) if scalar.kind == ScalarKind::Float => Ok(UniformType::Float),
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
        _ => Err(ShaderError::TypeMismatch("Unsupported type".into())),
    }
}
