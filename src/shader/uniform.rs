use crate::error::ShaderError;
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
            Self::Struct((fields, span)) => {
                // Allocate the full struct span and lay out fields at their declared offsets.
                // This avoids any mismatch with naga's reported span and ensures zero-padding
                // for gaps, which is important for strict backends (e.g., Dawn/WebGPU).
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

// WASM conversions

#[cfg(wasm)]
impl TryFrom<wasm_bindgen::JsValue> for UniformData {
    type Error = crate::error::ShaderError;

    fn try_from(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::{Array, Float32Array, Int32Array, Uint32Array};
        use wasm_bindgen::JsCast;

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

        Err(crate::error::ShaderError::TypeMismatch(
            "Cannot convert JavaScript value to UniformData".into(),
        ))
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Float32Array> for UniformData {
    type Error = crate::error::ShaderError;

    fn try_from(arr: &js_sys::Float32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 16 {
            return Err(crate::error::ShaderError::TypeMismatch(format!(
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
            _ => Err(crate::error::ShaderError::TypeMismatch(format!(
                "Unsupported Float32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Int32Array> for UniformData {
    type Error = crate::error::ShaderError;

    fn try_from(arr: &js_sys::Int32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 4 {
            return Err(crate::error::ShaderError::TypeMismatch(format!(
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
            _ => Err(crate::error::ShaderError::TypeMismatch(format!(
                "Unsupported Int32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Uint32Array> for UniformData {
    type Error = crate::error::ShaderError;

    fn try_from(arr: &js_sys::Uint32Array) -> Result<Self, Self::Error> {
        let len = arr.length() as usize;
        if len > 4 {
            return Err(crate::error::ShaderError::TypeMismatch(format!(
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
            _ => Err(crate::error::ShaderError::TypeMismatch(format!(
                "Unsupported Uint32Array length: {}",
                len
            ))),
        }
    }
}

#[cfg(wasm)]
impl TryFrom<&js_sys::Array> for UniformData {
    type Error = crate::error::ShaderError;

    fn try_from(array: &js_sys::Array) -> Result<Self, Self::Error> {
        let length = array.length();
        if length == 0 {
            return Err(crate::error::ShaderError::TypeMismatch(
                "Empty array".into(),
            ));
        }
        if length > 16 {
            return Err(crate::error::ShaderError::TypeMismatch(format!(
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
                    return Err(crate::error::ShaderError::TypeMismatch(
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
            _ => Err(crate::error::ShaderError::TypeMismatch(format!(
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

            UniformData::Texture(handle) => wasm_bindgen::JsValue::from_f64(handle as f64),

            // For complex types, return as regular JS arrays/objects
            UniformData::Array(items) => {
                let arr = Array::new();
                for (data, _, _) in items {
                    arr.push(&data.into());
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
        }
    }
}
