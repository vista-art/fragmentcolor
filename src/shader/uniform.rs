use crate::shader::error::ShaderError;
use naga::{Module, ScalarKind, Type, TypeInner, VectorSize};

#[cfg(python)]
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

#[cfg_attr(python, derive(FromPyObject, IntoPyObject))]
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
    Texture(crate::texture::TextureMeta),
    Sampler(crate::texture::SamplerInfo),
    // Array: (type, count, stride)
    Array(Vec<(UniformData, u32, u32)>),
    // Struct: name -> ((offset, name, field), struct_size)
    Struct((Vec<(u32, String, UniformData)>, u32)),
    // Storage buffer: (inner shape, total size/span, access flags)
    // Stored as a Vec to avoid infinite recursion (Box doesn't implement FromPyObject)
    Storage(Vec<(UniformData, u32, StorageAccess)>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(python, pyclass)]
pub enum StorageAccess {
    Read,
    Write,
    Atomic,
    ReadWrite,
    AtomicRead,
    AtomicWrite,
    AtomicReadWrite,
}

impl StorageAccess {
    pub(crate) fn is_readonly(&self) -> bool {
        matches!(
            self,
            StorageAccess::Read | StorageAccess::Atomic | StorageAccess::AtomicRead
        )
    }
}

impl From<naga::StorageAccess> for StorageAccess {
    fn from(value: naga::StorageAccess) -> Self {
        use naga::StorageAccess as SA;

        let load = value.contains(SA::LOAD);
        let store = value.contains(SA::STORE);
        let atomic = value.contains(SA::ATOMIC);

        match (load, store, atomic) {
            (true, false, false) => StorageAccess::Read,
            (false, true, false) => StorageAccess::Write,
            (false, false, true) => StorageAccess::Atomic,
            (true, true, false) => StorageAccess::ReadWrite,
            (true, false, true) => StorageAccess::AtomicRead,
            (false, true, true) => StorageAccess::AtomicWrite,
            (true, true, true) => StorageAccess::AtomicReadWrite,
            (false, false, false) => StorageAccess::ReadWrite,
        }
    }
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
            Self::Texture(_m) => Vec::new(),
            Self::Sampler(_s) => Vec::new(),
            Self::Array(items) => {
                // Respect naga-provided stride when laying out arrays.
                // items holds a single (elem_ty, count, stride) tuple.
                let mut bytes = Vec::new();
                if let Some((elem, count, stride)) = items.first() {
                    let elem_bytes = elem.to_bytes();
                    let total = (*stride as usize).saturating_mul(*count as usize);
                    bytes.resize(total, 0);
                    for i in 0..*count as usize {
                        let start = i * (*stride as usize);
                        let end = start + elem_bytes.len();
                        if end <= bytes.len() {
                            bytes[start..end].copy_from_slice(&elem_bytes);
                        }
                    }
                }
                bytes
            }
            Self::Struct((fields, span)) => {
                // Allocate the full struct span and lay out fields at their declared offsets.
                let mut bytes = vec![0u8; *span as usize];
                for (offset, _name, field) in fields.iter() {
                    let data = field.to_bytes();
                    let start = *offset as usize;
                    let end = start + data.len();
                    if end <= bytes.len() {
                        bytes[start..end].copy_from_slice(&data);
                    }
                }
                bytes
            }
            Self::Storage(data) => {
                if let Some((inner, span, _access)) = &data.iter().next() {
                    // Flatten inner representation; ensure it matches the declared span.
                    let mut bytes = inner.to_bytes();
                    if bytes.len() < *span as usize {
                        bytes.resize(*span as usize, 0);
                    } else if bytes.len() > *span as usize {
                        bytes.truncate(*span as usize);
                    }
                    bytes
                } else {
                    Vec::new()
                }
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
            Self::Texture(_) => 0,
            Self::Sampler(_) => 0,
            Self::Array(items) => {
                if let Some((_elem, count, stride)) = items.first() {
                    stride * count
                } else {
                    0
                }
            }
            Self::Struct((_, size)) => *size,
            // Storage buffers do not contribute to the CPU-side uniform buffer; size is 0 here.
            Self::Storage(_) => 0,
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
                ));
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
                    ));
                }
            };
            let base_ty = convert_type(module, &module.types[*base])?;

            let item = (base_ty, size, *stride);
            Ok(UniformData::Array(vec![item]))
        }
        TypeInner::Sampler { comparison } => {
            Ok(UniformData::Sampler(crate::texture::SamplerInfo {
                comparison: *comparison,
            }))
        }
        TypeInner::Image {
            dim,
            arrayed,
            class,
        } => Ok(UniformData::Texture(crate::texture::TextureMeta {
            id: crate::texture::TextureId(0),
            dim: *dim,
            arrayed: *arrayed,
            class: *class,
        })),

        _ => Err(ShaderError::TypeMismatch("Unsupported type".into())),
    }
}

// --------------------------------------------------------
// Conversions from primitive types for public interfaces
// --------------------------------------------------------

// 1 element or scalar

impl From<bool> for UniformData {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

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

impl From<[i32; 2]> for UniformData {
    fn from(value: [i32; 2]) -> Self {
        Self::IVec2(value)
    }
}

impl From<UniformData> for [i32; 2] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::IVec2(v) => v,
            _ => [0; 2],
        }
    }
}

impl From<[u32; 2]> for UniformData {
    fn from(value: [u32; 2]) -> Self {
        Self::UVec2(value)
    }
}

impl From<UniformData> for [u32; 2] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::UVec2(v) => v,
            _ => [0; 2],
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

impl From<[i32; 3]> for UniformData {
    fn from(value: [i32; 3]) -> Self {
        Self::IVec3(value)
    }
}

impl From<UniformData> for [i32; 3] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::IVec3(v) => v,
            _ => [0; 3],
        }
    }
}

impl From<[u32; 3]> for UniformData {
    fn from(value: [u32; 3]) -> Self {
        Self::UVec3(value)
    }
}

impl From<UniformData> for [u32; 3] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::UVec3(v) => v,
            _ => [0; 3],
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

impl From<[i32; 4]> for UniformData {
    fn from(value: [i32; 4]) -> Self {
        Self::IVec4(value)
    }
}

impl From<UniformData> for [i32; 4] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::IVec4(v) => v,
            _ => [0; 4],
        }
    }
}

impl From<[u32; 4]> for UniformData {
    fn from(value: [u32; 4]) -> Self {
        Self::UVec4(value)
    }
}

impl From<UniformData> for [u32; 4] {
    fn from(data: UniformData) -> Self {
        match data {
            UniformData::UVec4(v) => v,
            _ => [0; 4],
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

// Matrices
impl From<[[f32; 2]; 2]> for UniformData {
    fn from(value: [[f32; 2]; 2]) -> Self {
        Self::Mat2(value)
    }
}

impl From<[[f32; 3]; 3]> for UniformData {
    fn from(value: [[f32; 3]; 3]) -> Self {
        Self::Mat3(value)
    }
}

impl From<[[f32; 4]; 4]> for UniformData {
    fn from(value: [[f32; 4]; 4]) -> Self {
        Self::Mat4(value)
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

// Reference-based forwards for inputs to UniformData
crate::impl_from_ref!(UniformData, bool);
crate::impl_from_ref!(UniformData, f32);
crate::impl_from_ref!(UniformData, [f32; 1]);
crate::impl_from_ref!(UniformData, i32);
crate::impl_from_ref!(UniformData, [i32; 1]);
crate::impl_from_ref!(UniformData, u32);
crate::impl_from_ref!(UniformData, [u32; 1]);
crate::impl_from_ref!(UniformData, [f32; 2]);
crate::impl_from_ref!(UniformData, [i32; 2]);
crate::impl_from_ref!(UniformData, [u32; 2]);
crate::impl_from_ref!(UniformData, (f32, f32));
crate::impl_from_ref!(UniformData, glam::Vec2);
crate::impl_from_ref!(UniformData, [f32; 3]);
crate::impl_from_ref!(UniformData, [i32; 3]);
crate::impl_from_ref!(UniformData, [u32; 3]);
crate::impl_from_ref!(UniformData, (f32, f32, f32));
crate::impl_from_ref!(UniformData, glam::Vec3);
crate::impl_from_ref!(UniformData, [f32; 4]);
crate::impl_from_ref!(UniformData, [i32; 4]);
crate::impl_from_ref!(UniformData, [u32; 4]);
crate::impl_from_ref!(UniformData, (f32, f32, f32, f32));
crate::impl_from_ref!(UniformData, glam::Vec4);
crate::impl_from_ref!(UniformData, [[f32; 2]; 2]);
crate::impl_from_ref!(UniformData, [[f32; 3]; 3]);
crate::impl_from_ref!(UniformData, [[f32; 4]; 4]);
crate::impl_from_ref!(UniformData, wgpu::Extent3d);

// Reference-based forwards for outputs from UniformData
crate::impl_from_ref!(f32, UniformData);
crate::impl_from_ref!([f32; 1], UniformData);
crate::impl_from_ref!(i32, UniformData);
crate::impl_from_ref!([i32; 1], UniformData);
crate::impl_from_ref!(u32, UniformData);
crate::impl_from_ref!([u32; 1], UniformData);
crate::impl_from_ref!([f32; 2], UniformData);
crate::impl_from_ref!([i32; 2], UniformData);
crate::impl_from_ref!([u32; 2], UniformData);
crate::impl_from_ref!((f32, f32), UniformData);
crate::impl_from_ref!(glam::Vec2, UniformData);
crate::impl_from_ref!([f32; 3], UniformData);
crate::impl_from_ref!([i32; 3], UniformData);
crate::impl_from_ref!([u32; 3], UniformData);
crate::impl_from_ref!((f32, f32, f32), UniformData);
crate::impl_from_ref!(glam::Vec3, UniformData);
crate::impl_from_ref!([f32; 4], UniformData);
crate::impl_from_ref!([i32; 4], UniformData);
crate::impl_from_ref!([u32; 4], UniformData);
crate::impl_from_ref!((f32, f32, f32, f32), UniformData);
crate::impl_from_ref!(glam::Vec4, UniformData);
crate::impl_from_ref!(wgpu::Extent3d, UniformData);

// WASM conversions

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Int32Array, Uint32Array};
        use wasm_bindgen::JsCast;

        // Accept a Texture instance: convert to TextureMeta carrying id only.
        if let Some(tex) = value.dyn_ref::<crate::texture::Texture>() {
            let meta = crate::texture::TextureMeta::with_id_only(tex.id.clone());
            return Ok(UniformData::Texture(meta));
        }

        if let Some(arr) = value.dyn_ref::<Float32Array>() {
            return arr.try_into();
        }
        if let Some(arr) = value.dyn_ref::<Int32Array>() {
            return arr.try_into();
        }
        if let Some(arr) = value.dyn_ref::<Uint32Array>() {
            return arr.try_into();
        }
        if let Some(arr) = value.dyn_ref::<Array>() {
            return arr.try_into();
        }
        if let Some(n) = value.as_f64() {
            return Ok((n as f32).into());
        }
        if let Some(b) = value.as_bool() {
            return Ok(b.into());
        }

        Err(crate::shader::error::ShaderError::TypeMismatch(
            "Cannot convert JavaScript value to UniformData".into(),
        ))
    }
}

#[cfg(wasm)]
crate::impl_tryfrom_owned_via_ref!(
    UniformData,
    wasm_bindgen::JsValue,
    crate::shader::error::ShaderError
);

#[cfg(wasm)]
impl TryFrom<&js_sys::Float32Array> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(arr: &js_sys::Float32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 16 {
            return Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Float32Array length: {}",
                len
            )));
        }

        let mut buffer = [0.0f32; 16];
        let values = &mut buffer[..len];
        arr.copy_to(values);

        match len {
            1 => Ok(values[0].into()),
            2 => Ok([values[0], values[1]].into()),
            3 => Ok([values[0], values[1], values[2]].into()),
            4 => Ok([values[0], values[1], values[2], values[3]].into()),
            9 => {
                let mut mat = [[0.0; 3]; 3];
                for (i, chunk) in values.chunks_exact(3).enumerate() {
                    mat[i].copy_from_slice(chunk);
                }
                Ok(mat.into())
            }
            16 => {
                let mut mat = [[0.0; 4]; 4];
                for (i, chunk) in values.chunks_exact(4).enumerate() {
                    mat[i].copy_from_slice(chunk);
                }
                Ok(mat.into())
            }
            _ => Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Float32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Int32Array> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(arr: &js_sys::Int32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 4 {
            return Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Int32Array length: {}",
                len
            )));
        }

        let mut buffer = [0i32; 4];
        let values = &mut buffer[..len];
        arr.copy_to(values);

        match len {
            1 => Ok(values[0].into()),
            2 => Ok([values[0], values[1]].into()),
            3 => Ok([values[0], values[1], values[2]].into()),
            4 => Ok([values[0], values[1], values[2], values[3]].into()),
            _ => Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Int32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Uint32Array> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(arr: &js_sys::Uint32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 4 {
            return Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Uint32Array length: {}",
                len
            )));
        }

        let mut buffer = [0u32; 4];
        let values = &mut buffer[..len];
        arr.copy_to(values);

        match len {
            1 => Ok(values[0].into()),
            2 => Ok([values[0], values[1]].into()),
            3 => Ok([values[0], values[1], values[2]].into()),
            4 => Ok([values[0], values[1], values[2], values[3]].into()),
            _ => Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported Uint32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Array> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(array: &js_sys::Array) -> Result<Self, Self::Error> {
        let length = array.length();
        if length == 0 {
            return Err(crate::shader::error::ShaderError::TypeMismatch(
                "Empty array".into(),
            ));
        }
        if length > 16 {
            return Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported array length: {}",
                length
            )));
        }

        let mut buffer = [0.0f64; 16];
        let values = &mut buffer[..length as usize];

        for (i, val) in values.iter_mut().enumerate() {
            match array.get(i as u32).as_f64() {
                Some(n) => {
                    *val = n;
                }
                None => {
                    return Err(crate::shader::error::ShaderError::TypeMismatch(
                        "Array contains non-numeric values".into(),
                    ));
                }
            }
        }

        // Always treat plain JS arrays as float-based uniforms by default
        let mut float_buffer = [0.0f32; 16];
        let floats = &mut float_buffer[..values.len()];
        for (i, v) in values.iter().enumerate() {
            floats[i] = *v as f32;
        }

        match floats.len() {
            1 => Ok(floats[0].into()),
            2 => Ok([floats[0], floats[1]].into()),
            3 => Ok([floats[0], floats[1], floats[2]].into()),
            4 => Ok([floats[0], floats[1], floats[2], floats[3]].into()),
            9 => {
                let mut mat = [[0.0; 3]; 3];
                for (i, chunk) in floats.chunks_exact(3).enumerate() {
                    mat[i].copy_from_slice(chunk);
                }
                Ok(mat.into())
            }
            16 => {
                let mut mat = [[0.0; 4]; 4];
                for (i, chunk) in floats.chunks_exact(4).enumerate() {
                    mat[i].copy_from_slice(chunk);
                }
                Ok(mat.into())
            }
            _ => Err(crate::shader::error::ShaderError::TypeMismatch(format!(
                "Unsupported array length: {}",
                floats.len()
            ))),
        }
    }
}

#[cfg(wasm)]
impl From<UniformData> for wasm_bindgen::JsValue {
    fn from(data: UniformData) -> Self {
        use js_sys::{Array, Float32Array, Int32Array, Object, Reflect, Uint32Array};

        match data {
            UniformData::Bool(b) => wasm_bindgen::JsValue::from_bool(b),
            UniformData::Float(f) => wasm_bindgen::JsValue::from_f64(f as f64),
            UniformData::Int(i) => wasm_bindgen::JsValue::from_f64(i as f64),
            UniformData::UInt(u) => wasm_bindgen::JsValue::from_f64(u as f64),

            UniformData::Vec2(v) => {
                let arr = Float32Array::new_with_length(2);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::Vec3(v) => {
                let arr = Float32Array::new_with_length(3);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::Vec4(v) => {
                let arr = Float32Array::new_with_length(4);
                arr.copy_from(&v);
                arr.into()
            }

            UniformData::IVec2(v) => {
                let arr = Int32Array::new_with_length(2);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::IVec3(v) => {
                let arr = Int32Array::new_with_length(3);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::IVec4(v) => {
                let arr = Int32Array::new_with_length(4);
                arr.copy_from(&v);
                arr.into()
            }

            UniformData::UVec2(v) => {
                let arr = Uint32Array::new_with_length(2);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::UVec3(v) => {
                let arr = Uint32Array::new_with_length(3);
                arr.copy_from(&v);
                arr.into()
            }
            UniformData::UVec4(v) => {
                let arr = Uint32Array::new_with_length(4);
                arr.copy_from(&v);
                arr.into()
            }

            UniformData::Mat2(m) => {
                let flat: Vec<f32> = m.iter().flat_map(|row| row.iter()).copied().collect();
                let arr = Float32Array::new_with_length(4);
                arr.copy_from(&flat);
                arr.into()
            }
            UniformData::Mat3(m) => {
                let flat: Vec<f32> = m.iter().flat_map(|row| row.iter()).copied().collect();
                let arr = Float32Array::new_with_length(9);
                arr.copy_from(&flat);
                arr.into()
            }
            UniformData::Mat4(m) => {
                let flat: Vec<f32> = m.iter().flat_map(|row| row.iter()).copied().collect();
                let arr = Float32Array::new_with_length(16);
                arr.copy_from(&flat);
                arr.into()
            }

            UniformData::Texture(meta) => wasm_bindgen::JsValue::from_f64(meta.id.0 as f64),

            UniformData::Sampler(info) => {
                let obj = Object::new();
                let _ = Reflect::set(
                    &obj,
                    &wasm_bindgen::JsValue::from_str("comparison"),
                    &wasm_bindgen::JsValue::from_bool(info.comparison),
                );
                obj.into()
            }

            // For complex types, return as regular JS arrays/objects
            UniformData::Array(items) => {
                let mut arr = Array::new();
                for (item, count, _stride) in items {
                    let item_js: wasm_bindgen::JsValue = item.into();
                    for _ in 0..count {
                        arr.push(&item_js);
                    }
                }
                arr.into()
            }
            UniformData::Struct((fields, _)) => {
                let obj = Object::new();
                for (_, name, data) in fields {
                    let _ =
                        Reflect::set(&obj, &wasm_bindgen::JsValue::from_str(&name), &data.into());
                }
                obj.into()
            }
            UniformData::Storage(_) => wasm_bindgen::JsValue::UNDEFINED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::StorageAccess as SA;

    // Story: UniformData to_bytes/size layouts for Struct, Array, and Storage obey spans/strides.
    #[test]
    fn uniformdata_layouts_struct_array_storage() {
        // Struct with two fields at offsets 0 and 16, span 32
        let s = UniformData::Struct((
            vec![
                (0, "a".into(), UniformData::Vec4([1.0, 2.0, 3.0, 4.0])),
                (16, "b".into(), UniformData::Vec2([9.0, 8.0])),
            ],
            32,
        ));
        let bytes = s.to_bytes();
        assert_eq!(bytes.len(), 32);
        let a: [f32; 4] = bytemuck::cast_slice(&bytes[0..16]).try_into().unwrap();
        assert_eq!(a, [1.0, 2.0, 3.0, 4.0]);
        let b: [f32; 2] = bytemuck::cast_slice(&bytes[16..24]).try_into().unwrap();
        assert_eq!(b, [9.0, 8.0]);

        // Array of vec4 with count 2 and stride 16
        let arr = UniformData::Array(vec![(UniformData::Vec4([0.5, 0.5, 0.5, 0.5]), 2, 16)]);
        let bytes = arr.to_bytes();
        assert_eq!(bytes.len(), 32);
        // first element at 0..16
        let e0: [f32; 4] = bytemuck::cast_slice(&bytes[0..16]).try_into().unwrap();
        assert_eq!(e0, [0.5, 0.5, 0.5, 0.5]);
        // second element at 16..32
        let e1: [f32; 4] = bytemuck::cast_slice(&bytes[16..32]).try_into().unwrap();
        assert_eq!(e1, [0.5, 0.5, 0.5, 0.5]);

        // Storage wraps a shape with span and clamps/truncates to the span
        let stor = UniformData::Storage(vec![(
            UniformData::Vec4([1.0, 2.0, 3.0, 4.0]),
            8,
            StorageAccess::Read,
        )]);
        let bytes = stor.to_bytes();
        assert_eq!(bytes.len(), 8);
        // size() for Struct/Array reflects spans, Storage returns 0 for CPU uniform upload
        assert_eq!(s.size(), 32);
        assert_eq!(arr.size(), 32);
        assert_eq!(stor.size(), 0);
    }

    #[test]
    fn storage_access_combinations() {
        assert_eq!(StorageAccess::from(SA::LOAD), StorageAccess::Read);
        assert_eq!(StorageAccess::from(SA::STORE), StorageAccess::Write);
        assert_eq!(StorageAccess::from(SA::ATOMIC), StorageAccess::Atomic);
        assert_eq!(
            StorageAccess::from(SA::LOAD | SA::STORE),
            StorageAccess::ReadWrite
        );
        assert_eq!(
            StorageAccess::from(SA::LOAD | SA::ATOMIC),
            StorageAccess::AtomicRead
        );
        assert_eq!(
            StorageAccess::from(SA::STORE | SA::ATOMIC),
            StorageAccess::AtomicWrite
        );
        assert_eq!(
            StorageAccess::from(SA::LOAD | SA::STORE | SA::ATOMIC),
            StorageAccess::AtomicReadWrite
        );
    }

    // Story: Scalar, vector and matrix to_bytes lengths and size() are as expected.
    #[test]
    fn scalar_vector_matrix_sizes_and_bytes() {
        assert_eq!(UniformData::Float(1.0).to_bytes().len(), 4);
        assert_eq!(UniformData::Vec3([1.0, 2.0, 3.0]).to_bytes().len(), 12);
        assert_eq!(UniformData::Mat2([[0.0; 2]; 2]).to_bytes().len(), 16);
        assert_eq!(UniformData::Mat3([[0.0; 3]; 3]).to_bytes().len(), 36);
        assert_eq!(UniformData::Mat4([[0.0; 4]; 4]).to_bytes().len(), 64);
        assert_eq!(UniformData::Vec3([0.0; 3]).size(), 12);
        assert_eq!(UniformData::Mat3([[0.0; 3]; 3]).size(), 36);
    }

    // Story: Array stride padding is honored; elements land at multiples of stride with zeros in between.
    #[test]
    fn array_stride_padding_is_honored() {
        // Two vec2 (8 bytes each) with stride 16 -> 32 total
        let u = UniformData::Array(vec![(UniformData::Vec2([2.0, 4.0]), 2, 16)]);
        let bytes = u.to_bytes();
        assert_eq!(bytes.len(), 32);
        let e0: [f32; 2] = bytemuck::cast_slice(&bytes[0..8]).try_into().unwrap();
        let e1: [f32; 2] = bytemuck::cast_slice(&bytes[16..24]).try_into().unwrap();
        assert_eq!(e0, [2.0, 4.0]);
        assert_eq!(e1, [2.0, 4.0]);
        // Padding region between 8..16 should be zeros
        assert!(bytes[8..16].iter().all(|&b| b == 0));
    }

    // Story: Extent3d conversions from various uniform shapes behave consistently.
    #[test]
    fn extent3d_conversions_from_uniforms() {
        use wgpu::Extent3d;
        let e: Extent3d = UniformData::UVec3([10, 20, 3]).into();
        assert_eq!((e.width, e.height, e.depth_or_array_layers), (10, 20, 3));
        let e2: Extent3d = UniformData::Vec4([9.0, 8.0, 7.0, 6.0]).into();
        assert_eq!((e2.width, e2.height, e2.depth_or_array_layers), (9, 8, 7));
        let e3: Extent3d = UniformData::Bool(true).into();
        assert_eq!((e3.width, e3.height, e3.depth_or_array_layers), (0, 0, 0));
    }

    // Story: Round-trips for scalar wrappers (f32/i32/u32 and single-element arrays)
    #[test]
    fn scalar_round_trips() {
        let f: f32 = UniformData::from(3.5f32).into();
        assert_eq!(f, 3.5);
        let i: i32 = UniformData::from(7i32).into();
        assert_eq!(i, 7);
        let u: u32 = UniformData::from(9u32).into();
        assert_eq!(u, 9);
        let a1f: [f32; 1] = UniformData::from([1.25f32]).into();
        assert_eq!(a1f, [1.25]);
        let a1i: [i32; 1] = UniformData::from([5i32]).into();
        assert_eq!(a1i, [5]);
        let a1u: [u32; 1] = UniformData::from([6u32]).into();
        assert_eq!(a1u, [6]);
    }
}
