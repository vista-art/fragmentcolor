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
    // Push constant root: (inner_shape, span)
    // Vec used for PyO3 compatibility; Same pattern as Storage.
    PushConstant(Vec<(UniformData, u32)>),
    // Raw bytes (opaque); intended for storage-root or push-root updates via Shader::set("root", bytes)
    Bytes(Vec<u8>),
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
            Self::PushConstant(data) => {
                if let Some((inner, span)) = &data.iter().next() {
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
            Self::Bytes(b) => b.clone(),
        }
    }

    pub(crate) fn size(&self) -> u32 {
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
            // Push constants do not contribute to the CPU-side uniform buffer aggregate; return 0 here.
            Self::PushConstant(_) => 0,
            Self::Bytes(b) => b.len() as u32,
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
        TypeInner::Vector { size, scalar, .. } => match scalar.kind {
            ScalarKind::Float => Ok(match size {
                VectorSize::Bi => UniformData::Vec2([0.0; 2]),
                VectorSize::Tri => UniformData::Vec3([0.0; 3]),
                VectorSize::Quad => UniformData::Vec4([0.0; 4]),
            }),
            ScalarKind::Uint => Ok(match size {
                VectorSize::Bi => UniformData::UVec2([0; 2]),
                VectorSize::Tri => UniformData::UVec3([0; 3]),
                VectorSize::Quad => UniformData::UVec4([0; 4]),
            }),
            ScalarKind::Sint => Ok(match size {
                VectorSize::Bi => UniformData::IVec2([0; 2]),
                VectorSize::Tri => UniformData::IVec3([0; 3]),
                VectorSize::Quad => UniformData::IVec4([0; 4]),
            }),
            _ => Err(ShaderError::TypeMismatch(
                "Unsupported vector scalar type".into(),
            )),
        },
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

crate::impl_from_into_with_refs!(
    UniformData,
    bool,
    |d: UniformData| match d {
        UniformData::Bool(v) => v,
        _ => false,
    },
    |b: bool| UniformData::Bool(b)
);

crate::impl_from_into_with_refs!(
    UniformData,
    f32,
    |d: UniformData| match d {
        UniformData::Float(v) => v,
        _ => 0.0,
    },
    |v: f32| UniformData::Float(v)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [f32; 1],
    |d: UniformData| match d {
        UniformData::Float(v) => [v],
        _ => [0.0],
    },
    |a: [f32; 1]| UniformData::Float(a[0])
);

crate::impl_from_into_with_refs!(
    UniformData,
    i32,
    |d: UniformData| match d {
        UniformData::Int(v) => v,
        _ => 0,
    },
    |v: i32| UniformData::Int(v)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [i32; 1],
    |d: UniformData| match d {
        UniformData::Int(v) => [v],
        _ => [0],
    },
    |a: [i32; 1]| UniformData::Int(a[0])
);

crate::impl_from_into_with_refs!(
    UniformData,
    u32,
    |d: UniformData| match d {
        UniformData::UInt(v) => v,
        _ => 0,
    },
    |v: u32| UniformData::UInt(v)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [u32; 1],
    |d: UniformData| match d {
        UniformData::UInt(v) => [v],
        _ => [0],
    },
    |a: [u32; 1]| UniformData::UInt(a[0])
);

// 2 elements

crate::impl_from_into_with_refs!(
    UniformData,
    [f32; 2],
    |d: UniformData| match d {
        UniformData::Vec2(v) => v,
        _ => [0.0; 2],
    },
    |a: [f32; 2]| UniformData::Vec2(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [i32; 2],
    |d: UniformData| match d {
        UniformData::IVec2(v) => v,
        _ => [0; 2],
    },
    |a: [i32; 2]| UniformData::IVec2(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [u32; 2],
    |d: UniformData| match d {
        UniformData::UVec2(v) => v,
        _ => [0; 2],
    },
    |a: [u32; 2]| UniformData::UVec2(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    (f32, f32),
    |d: UniformData| match d {
        UniformData::Vec2(v) => (v[0], v[1]),
        _ => (0.0, 0.0),
    },
    |t: (f32, f32)| UniformData::Vec2([t.0, t.1])
);

crate::impl_from_into_with_refs!(
    UniformData,
    glam::Vec2,
    |d: UniformData| match d {
        UniformData::Vec2(v) => glam::Vec2::from(v),
        _ => glam::Vec2::ZERO,
    },
    |v: glam::Vec2| UniformData::Vec2(v.to_array())
);

// 3 elements

crate::impl_from_into_with_refs!(
    UniformData,
    [f32; 3],
    |d: UniformData| match d {
        UniformData::Vec3(v) => v,
        _ => [0.0; 3],
    },
    |a: [f32; 3]| UniformData::Vec3(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [i32; 3],
    |d: UniformData| match d {
        UniformData::IVec3(v) => v,
        _ => [0; 3],
    },
    |a: [i32; 3]| UniformData::IVec3(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [u32; 3],
    |d: UniformData| match d {
        UniformData::UVec3(v) => v,
        _ => [0; 3],
    },
    |a: [u32; 3]| UniformData::UVec3(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    (f32, f32, f32),
    |d: UniformData| match d {
        UniformData::Vec3(v) => (v[0], v[1], v[2]),
        _ => (0.0, 0.0, 0.0),
    },
    |t: (f32, f32, f32)| UniformData::Vec3([t.0, t.1, t.2])
);

crate::impl_from_into_with_refs!(
    UniformData,
    glam::Vec3,
    |d: UniformData| match d {
        UniformData::Vec3(v) => glam::Vec3::from(v),
        _ => glam::Vec3::ZERO,
    },
    |v: glam::Vec3| UniformData::Vec3(v.to_array())
);

// 4 elements

crate::impl_from_into_with_refs!(
    UniformData,
    [f32; 4],
    |d: UniformData| match d {
        UniformData::Vec4(v) => v,
        _ => [0.0; 4],
    },
    |a: [f32; 4]| UniformData::Vec4(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [i32; 4],
    |d: UniformData| match d {
        UniformData::IVec4(v) => v,
        _ => [0; 4],
    },
    |a: [i32; 4]| UniformData::IVec4(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [u32; 4],
    |d: UniformData| match d {
        UniformData::UVec4(v) => v,
        _ => [0; 4],
    },
    |a: [u32; 4]| UniformData::UVec4(a)
);

crate::impl_from_into_with_refs!(
    UniformData,
    (f32, f32, f32, f32),
    |d: UniformData| match d {
        UniformData::Vec4(v) => (v[0], v[1], v[2], v[3]),
        _ => (0.0, 0.0, 0.0, 0.0),
    },
    |t: (f32, f32, f32, f32)| UniformData::Vec4([t.0, t.1, t.2, t.3])
);

crate::impl_from_into_with_refs!(
    UniformData,
    glam::Vec4,
    |d: UniformData| match d {
        UniformData::Vec4(v) => glam::Vec4::from(v),
        _ => glam::Vec4::ZERO,
    },
    |v: glam::Vec4| UniformData::Vec4(v.to_array())
);

// 2 elements

// 3 elements

// 4 elements

// Matrices
crate::impl_from_into_with_refs!(
    UniformData,
    [[f32; 2]; 2],
    |d: UniformData| match d {
        UniformData::Mat2(m) => m,
        _ => [[0.0; 2]; 2],
    },
    |m: [[f32; 2]; 2]| UniformData::Mat2(m)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [[f32; 3]; 3],
    |d: UniformData| match d {
        UniformData::Mat3(m) => m,
        _ => [[0.0; 3]; 3],
    },
    |m: [[f32; 3]; 3]| UniformData::Mat3(m)
);

crate::impl_from_into_with_refs!(
    UniformData,
    [[f32; 4]; 4],
    |d: UniformData| match d {
        UniformData::Mat4(m) => m,
        _ => [[0.0; 4]; 4],
    },
    |m: [[f32; 4]; 4]| UniformData::Mat4(m)
);

crate::impl_from_into_with_refs!(
    UniformData,
    wgpu::Extent3d,
    |data: UniformData| match data {
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
    },
    |value: wgpu::Extent3d| UniformData::UVec3([
        value.width,
        value.height,
        value.depth_or_array_layers,
    ])
);

// Simple conversions for byte slices
impl From<&[u8]> for UniformData {
    fn from(v: &[u8]) -> Self {
        UniformData::Bytes(v.to_vec())
    }
}
impl From<Vec<u8>> for UniformData {
    fn from(v: Vec<u8>) -> Self {
        UniformData::Bytes(v)
    }
}

// WASM conversions

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for UniformData {
    type Error = crate::shader::error::ShaderError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Int32Array, Uint32Array};
        use wasm_bindgen::JsCast;

        {
            use js_sys::Reflect;
            use wasm_bindgen::convert::RefFromWasmAbi;
            let key = wasm_bindgen::JsValue::from_str("__wbg_ptr");
            if let Ok(ptr) = Reflect::get(value, &key) {
                if let Some(id) = ptr.as_f64() {
                    let anchor: <crate::texture::Texture as RefFromWasmAbi>::Anchor = unsafe {
                        <crate::texture::Texture as RefFromWasmAbi>::ref_from_abi(id as u32)
                    };
                    let tex = anchor.clone();
                    let meta = crate::texture::TextureMeta::with_id_only(tex.id.clone());
                    return Ok(UniformData::Texture(meta));
                }
            }
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

            UniformData::Array(items) => {
                let arr = Array::new();
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
            UniformData::PushConstant(_) => wasm_bindgen::JsValue::UNDEFINED,

            UniformData::Bytes(b) => {
                let arr = Uint32Array::new_with_length(b.len() as u32);
                let byte_u32: Vec<u32> = b.iter().map(|&byte| byte as u32).collect();
                arr.copy_from(&byte_u32);
                arr.into()
            }
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

    // Story: Vector conversions cover arrays, tuples, and glam types for f32 vectors.
    #[test]
    fn vector_conversions_roundtrip_vec2_vec3_vec4() {
        // Vec2
        let u = UniformData::from([1.0f32, 2.0]);
        let a: [f32; 2] = u.clone().into();
        assert_eq!(a, [1.0, 2.0]);
        let t: (f32, f32) = u.clone().into();
        assert_eq!(t, (1.0, 2.0));
        let gv: glam::Vec2 = u.clone().into();
        assert_eq!(gv, glam::vec2(1.0, 2.0));

        // Vec3
        let u = UniformData::from([3.0f32, 4.0, 5.0]);
        let a: [f32; 3] = u.clone().into();
        assert_eq!(a, [3.0, 4.0, 5.0]);
        let t: (f32, f32, f32) = u.clone().into();
        assert_eq!(t, (3.0, 4.0, 5.0));
        let gv: glam::Vec3 = u.clone().into();
        assert_eq!(gv, glam::vec3(3.0, 4.0, 5.0));

        // Vec4
        let u = UniformData::from([6.0f32, 7.0, 8.0, 9.0]);
        let a: [f32; 4] = u.clone().into();
        assert_eq!(a, [6.0, 7.0, 8.0, 9.0]);
        let t: (f32, f32, f32, f32) = u.clone().into();
        assert_eq!(t, (6.0, 7.0, 8.0, 9.0));
        let gv: glam::Vec4 = u.clone().into();
        assert_eq!(gv, glam::vec4(6.0, 7.0, 8.0, 9.0));
    }

    // Story: Integer vector conversions for IVecN and UVecN arrays work round-trip.
    #[test]
    fn integer_vector_conversions_roundtrip() {
        let iu = UniformData::from([1i32, -2]);
        let ia: [i32; 2] = iu.clone().into();
        assert_eq!(ia, [1, -2]);

        let iu3 = UniformData::from([1i32, 2, 3]);
        let ia3: [i32; 3] = iu3.clone().into();
        assert_eq!(ia3, [1, 2, 3]);

        let uu = UniformData::from([10u32, 20u32, 30u32, 40u32]);
        let ua: [u32; 4] = uu.clone().into();
        assert_eq!(ua, [10, 20, 30, 40]);
    }

    // Story: Matrix conversions round-trip across Mat2/Mat3/Mat4.
    #[test]
    fn matrix_roundtrips() {
        let m2_src = [[1.0f32, 2.0], [3.0, 4.0]];
        let um2: UniformData = m2_src.into();
        let m2: [[f32; 2]; 2] = um2.into();
        assert_eq!(m2, m2_src);

        let m3_src = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let um3: UniformData = m3_src.into();
        let m3: [[f32; 3]; 3] = um3.into();
        assert_eq!(m3, m3_src);

        let m4_src = [
            [1.0f32, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        let um4: UniformData = m4_src.into();
        let m4: [[f32; 4]; 4] = um4.into();
        assert_eq!(m4, m4_src);
    }

    // Story: Additional Extent3d conversions from Vec2/IVec2/IVec3/UVec4.
    #[test]
    fn extent3d_more_conversions() {
        use wgpu::Extent3d;
        let e: Extent3d = UniformData::Vec2([100.0, 200.0]).into();
        assert_eq!((e.width, e.height, e.depth_or_array_layers), (100, 200, 1));

        let e: Extent3d = UniformData::IVec2([3, 4]).into();
        assert_eq!((e.width, e.height, e.depth_or_array_layers), (3, 4, 1));

        let e: Extent3d = UniformData::IVec3([5, 6, 7]).into();
        assert_eq!((e.width, e.height, e.depth_or_array_layers), (5, 6, 7));

        let e: Extent3d = UniformData::UVec4([9, 8, 7, 6]).into();
        assert_eq!((e.width, e.height, e.depth_or_array_layers), (9, 8, 7));
    }

    // Story: StorageAccess::is_readonly correctly classifies read-only variants.
    #[test]
    fn storage_access_readonly_flags() {
        assert!(StorageAccess::Read.is_readonly());
        assert!(StorageAccess::Atomic.is_readonly());
        assert!(StorageAccess::AtomicRead.is_readonly());
        assert!(!StorageAccess::Write.is_readonly());
        assert!(!StorageAccess::ReadWrite.is_readonly());
        assert!(!StorageAccess::AtomicWrite.is_readonly());
        assert!(!StorageAccess::AtomicReadWrite.is_readonly());
    }

    // Story: Bytes conversions from &[u8] and Vec<u8] round-trip size and payload.
    #[test]
    fn bytes_conversions_size_and_bytes() {
        let b = vec![1u8, 2, 3, 4, 5];
        let u1: UniformData = (&b[..]).into();
        assert_eq!(u1.size(), b.len() as u32);
        assert_eq!(u1.to_bytes(), b);

        let b2 = vec![9u8, 8, 7];
        let u2: UniformData = b2.clone().into();
        assert_eq!(u2.size(), 3);
        assert_eq!(u2.to_bytes(), b2);
    }

    // Story: PushConstant to_bytes respects span and size() is zero.
    #[test]
    fn push_constant_to_bytes_and_size() {
        let u = UniformData::PushConstant(vec![(UniformData::Vec4([10.0, 20.0, 30.0, 40.0]), 8)]);
        let bytes = u.to_bytes();
        assert_eq!(bytes.len(), 8);
        let first_two: [f32; 2] = bytemuck::cast_slice(&bytes).try_into().unwrap();
        assert_eq!(first_two, [10.0, 20.0]);
        assert_eq!(u.size(), 0);
    }

    // Story: Sampler to_bytes is empty and size is zero.
    #[test]
    fn sampler_to_bytes_and_size() {
        let u = UniformData::Sampler(crate::texture::SamplerInfo { comparison: true });
        assert!(u.to_bytes().is_empty());
        assert_eq!(u.size(), 0);
    }
}
