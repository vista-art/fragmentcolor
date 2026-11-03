//! How a shader declares a texture: `TextureMeta`, and the dimension,
//! class, sample kind, storage format and access it reflects.

#[cfg(python)]
use pyo3::prelude::*;

use crate::texture::TextureId;

/// Image-view dimension. Mirrors `naga::ImageDimension`.
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDim {
    D1,
    D2,
    D3,
    Cube,
}

impl TextureDim {
    /// The `TextureDim` for naga's `ImageDimension`.
    pub(crate) fn from_naga(value: naga::ImageDimension) -> Self {
        match value {
            naga::ImageDimension::D1 => TextureDim::D1,
            naga::ImageDimension::D2 => TextureDim::D2,
            naga::ImageDimension::D3 => TextureDim::D3,
            naga::ImageDimension::Cube => TextureDim::Cube,
        }
    }
}

/// Texel scalar kind for sampled textures. Mirrors the relevant `naga::ScalarKind`
/// variants. Naga's abstract numeric variants are folded into `Float` since they
/// cannot reach the runtime backends.
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureScalarKind {
    Sint,
    Uint,
    Float,
    Bool,
}

impl TextureScalarKind {
    /// The `TextureScalarKind` for naga's `ScalarKind`.
    pub(crate) fn from_naga(value: naga::ScalarKind) -> Self {
        match value {
            naga::ScalarKind::Sint => TextureScalarKind::Sint,
            naga::ScalarKind::Uint => TextureScalarKind::Uint,
            naga::ScalarKind::Float => TextureScalarKind::Float,
            naga::ScalarKind::Bool => TextureScalarKind::Bool,
            naga::ScalarKind::AbstractInt | naga::ScalarKind::AbstractFloat => {
                TextureScalarKind::Float
            }
        }
    }
}

/// Storage-texture format. Mirrors `naga::StorageFormat`.
///
/// Every variant binds as the `wgpu::TextureFormat` of the same name.
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureStorageFormat {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,

    R16Uint,
    R16Sint,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,

    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Float,
    Rgba8Unorm,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,

    Rgb10a2Uint,
    Rgb10a2Unorm,
    Rg11b10Ufloat,

    R64Uint,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Float,

    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,

    R16Unorm,
    R16Snorm,
    Rg16Unorm,
    Rg16Snorm,
    Rgba16Unorm,
    Rgba16Snorm,
}

impl TextureStorageFormat {
    /// The `TextureStorageFormat` for naga's `StorageFormat`.
    pub(crate) fn from_naga(value: naga::StorageFormat) -> Self {
        use naga::StorageFormat as F;
        match value {
            F::R8Unorm => TextureStorageFormat::R8Unorm,
            F::R8Snorm => TextureStorageFormat::R8Snorm,
            F::R8Uint => TextureStorageFormat::R8Uint,
            F::R8Sint => TextureStorageFormat::R8Sint,
            F::R16Uint => TextureStorageFormat::R16Uint,
            F::R16Sint => TextureStorageFormat::R16Sint,
            F::R16Float => TextureStorageFormat::R16Float,
            F::Rg8Unorm => TextureStorageFormat::Rg8Unorm,
            F::Rg8Snorm => TextureStorageFormat::Rg8Snorm,
            F::Rg8Uint => TextureStorageFormat::Rg8Uint,
            F::Rg8Sint => TextureStorageFormat::Rg8Sint,
            F::R32Uint => TextureStorageFormat::R32Uint,
            F::R32Sint => TextureStorageFormat::R32Sint,
            F::R32Float => TextureStorageFormat::R32Float,
            F::Rg16Uint => TextureStorageFormat::Rg16Uint,
            F::Rg16Sint => TextureStorageFormat::Rg16Sint,
            F::Rg16Float => TextureStorageFormat::Rg16Float,
            F::Rgba8Unorm => TextureStorageFormat::Rgba8Unorm,
            F::Rgba8Snorm => TextureStorageFormat::Rgba8Snorm,
            F::Rgba8Uint => TextureStorageFormat::Rgba8Uint,
            F::Rgba8Sint => TextureStorageFormat::Rgba8Sint,
            F::Bgra8Unorm => TextureStorageFormat::Bgra8Unorm,
            F::Rgb10a2Uint => TextureStorageFormat::Rgb10a2Uint,
            F::Rgb10a2Unorm => TextureStorageFormat::Rgb10a2Unorm,
            F::Rg11b10Ufloat => TextureStorageFormat::Rg11b10Ufloat,
            F::R64Uint => TextureStorageFormat::R64Uint,
            F::Rg32Uint => TextureStorageFormat::Rg32Uint,
            F::Rg32Sint => TextureStorageFormat::Rg32Sint,
            F::Rg32Float => TextureStorageFormat::Rg32Float,
            F::Rgba16Uint => TextureStorageFormat::Rgba16Uint,
            F::Rgba16Sint => TextureStorageFormat::Rgba16Sint,
            F::Rgba16Float => TextureStorageFormat::Rgba16Float,
            F::Rgba32Uint => TextureStorageFormat::Rgba32Uint,
            F::Rgba32Sint => TextureStorageFormat::Rgba32Sint,
            F::Rgba32Float => TextureStorageFormat::Rgba32Float,
            F::R16Unorm => TextureStorageFormat::R16Unorm,
            F::R16Snorm => TextureStorageFormat::R16Snorm,
            F::Rg16Unorm => TextureStorageFormat::Rg16Unorm,
            F::Rg16Snorm => TextureStorageFormat::Rg16Snorm,
            F::Rgba16Unorm => TextureStorageFormat::Rgba16Unorm,
            F::Rgba16Snorm => TextureStorageFormat::Rgba16Snorm,
        }
    }
}

impl TextureStorageFormat {
    /// The wgpu `TextureFormat` this `TextureStorageFormat` stands for.
    pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
        use wgpu::TextureFormat as F;
        match self {
            TextureStorageFormat::R8Unorm => F::R8Unorm,
            TextureStorageFormat::R8Snorm => F::R8Snorm,
            TextureStorageFormat::R8Uint => F::R8Uint,
            TextureStorageFormat::R8Sint => F::R8Sint,
            TextureStorageFormat::R16Uint => F::R16Uint,
            TextureStorageFormat::R16Sint => F::R16Sint,
            TextureStorageFormat::R16Float => F::R16Float,
            TextureStorageFormat::Rg8Unorm => F::Rg8Unorm,
            TextureStorageFormat::Rg8Snorm => F::Rg8Snorm,
            TextureStorageFormat::Rg8Uint => F::Rg8Uint,
            TextureStorageFormat::Rg8Sint => F::Rg8Sint,
            TextureStorageFormat::R32Uint => F::R32Uint,
            TextureStorageFormat::R32Sint => F::R32Sint,
            TextureStorageFormat::R32Float => F::R32Float,
            TextureStorageFormat::Rg16Uint => F::Rg16Uint,
            TextureStorageFormat::Rg16Sint => F::Rg16Sint,
            TextureStorageFormat::Rg16Float => F::Rg16Float,
            TextureStorageFormat::Rgba8Unorm => F::Rgba8Unorm,
            TextureStorageFormat::Rgba8Snorm => F::Rgba8Snorm,
            TextureStorageFormat::Rgba8Uint => F::Rgba8Uint,
            TextureStorageFormat::Rgba8Sint => F::Rgba8Sint,
            TextureStorageFormat::Bgra8Unorm => F::Bgra8Unorm,
            TextureStorageFormat::Rgb10a2Uint => F::Rgb10a2Uint,
            TextureStorageFormat::Rgb10a2Unorm => F::Rgb10a2Unorm,
            TextureStorageFormat::Rg11b10Ufloat => F::Rg11b10Ufloat,
            TextureStorageFormat::R64Uint => F::R64Uint,
            TextureStorageFormat::Rg32Uint => F::Rg32Uint,
            TextureStorageFormat::Rg32Sint => F::Rg32Sint,
            TextureStorageFormat::Rg32Float => F::Rg32Float,
            TextureStorageFormat::Rgba16Uint => F::Rgba16Uint,
            TextureStorageFormat::Rgba16Sint => F::Rgba16Sint,
            TextureStorageFormat::Rgba16Float => F::Rgba16Float,
            TextureStorageFormat::Rgba32Uint => F::Rgba32Uint,
            TextureStorageFormat::Rgba32Sint => F::Rgba32Sint,
            TextureStorageFormat::Rgba32Float => F::Rgba32Float,
            TextureStorageFormat::R16Unorm => F::R16Unorm,
            TextureStorageFormat::R16Snorm => F::R16Snorm,
            TextureStorageFormat::Rg16Unorm => F::Rg16Unorm,
            TextureStorageFormat::Rg16Snorm => F::Rg16Snorm,
            TextureStorageFormat::Rgba16Unorm => F::Rgba16Unorm,
            TextureStorageFormat::Rgba16Snorm => F::Rgba16Snorm,
        }
    }
}

/// Storage-texture access flags. Boolean mirror of the `naga::StorageAccess` bitflags.
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureStorageAccess {
    pub load: bool,
    pub store: bool,
    pub atomic: bool,
}

impl TextureStorageAccess {
    /// The `TextureStorageAccess` for naga's `StorageAccess`.
    pub(crate) fn from_naga(value: naga::StorageAccess) -> Self {
        use naga::StorageAccess as SA;
        TextureStorageAccess {
            load: value.contains(SA::LOAD),
            store: value.contains(SA::STORE),
            atomic: value.contains(SA::ATOMIC),
        }
    }
}

/// Image sub-class. Mirrors `naga::ImageClass`.
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureClass {
    Sampled {
        kind: TextureScalarKind,
        multi: bool,
    },
    Storage {
        format: TextureStorageFormat,
        access: TextureStorageAccess,
    },
    Depth {
        multi: bool,
    },
    External(),
}

impl TextureClass {
    /// The `TextureClass` for naga's `ImageClass`.
    pub(crate) fn from_naga(value: naga::ImageClass) -> Self {
        match value {
            naga::ImageClass::Sampled { kind, multi } => TextureClass::Sampled {
                kind: TextureScalarKind::from_naga(kind),
                multi,
            },
            naga::ImageClass::Storage { format, access } => TextureClass::Storage {
                format: TextureStorageFormat::from_naga(format),
                access: TextureStorageAccess::from_naga(access),
            },
            naga::ImageClass::Depth { multi } => TextureClass::Depth { multi },
            naga::ImageClass::External => TextureClass::External(),
        }
    }
}

// Metadata for textures parsed from shader source; users do not construct directly.
#[cfg_attr(python, pyo3::pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct TextureMeta {
    pub id: TextureId,
    pub dim: TextureDim,
    pub arrayed: bool,
    pub class: TextureClass,
    /// Whether the shader ever samples this texture (`textureSample*`). When false
    /// (only `textureLoad` / image-ops are used), the bind-group layout can request
    /// a non-filterable sample type, which unlocks formats like Rgba32Float as a
    /// sampled source without requiring the `FLOAT32_FILTERABLE` feature.
    pub sampled: bool,
}

/// A 2D float texture that a shader samples, named by `id`.
impl From<TextureId> for TextureMeta {
    fn from(id: TextureId) -> Self {
        TextureMeta {
            id,
            dim: TextureDim::D2,
            arrayed: false,
            class: TextureClass::Sampled {
                kind: TextureScalarKind::Float,
                multi: false,
            },
            sampled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl TextureDim {
        /// The naga `ImageDimension` this `TextureDim` stands for.
        fn to_naga(self) -> naga::ImageDimension {
            match self {
                TextureDim::D1 => naga::ImageDimension::D1,
                TextureDim::D2 => naga::ImageDimension::D2,
                TextureDim::D3 => naga::ImageDimension::D3,
                TextureDim::Cube => naga::ImageDimension::Cube,
            }
        }
    }

    impl TextureScalarKind {
        /// The naga `ScalarKind` this `TextureScalarKind` stands for.
        fn to_naga(self) -> naga::ScalarKind {
            match self {
                TextureScalarKind::Sint => naga::ScalarKind::Sint,
                TextureScalarKind::Uint => naga::ScalarKind::Uint,
                TextureScalarKind::Float => naga::ScalarKind::Float,
                TextureScalarKind::Bool => naga::ScalarKind::Bool,
            }
        }
    }

    impl TextureStorageFormat {
        /// The naga `StorageFormat` this `TextureStorageFormat` stands for.
        fn to_naga(self) -> naga::StorageFormat {
            use naga::StorageFormat as F;
            match self {
                TextureStorageFormat::R8Unorm => F::R8Unorm,
                TextureStorageFormat::R8Snorm => F::R8Snorm,
                TextureStorageFormat::R8Uint => F::R8Uint,
                TextureStorageFormat::R8Sint => F::R8Sint,
                TextureStorageFormat::R16Uint => F::R16Uint,
                TextureStorageFormat::R16Sint => F::R16Sint,
                TextureStorageFormat::R16Float => F::R16Float,
                TextureStorageFormat::Rg8Unorm => F::Rg8Unorm,
                TextureStorageFormat::Rg8Snorm => F::Rg8Snorm,
                TextureStorageFormat::Rg8Uint => F::Rg8Uint,
                TextureStorageFormat::Rg8Sint => F::Rg8Sint,
                TextureStorageFormat::R32Uint => F::R32Uint,
                TextureStorageFormat::R32Sint => F::R32Sint,
                TextureStorageFormat::R32Float => F::R32Float,
                TextureStorageFormat::Rg16Uint => F::Rg16Uint,
                TextureStorageFormat::Rg16Sint => F::Rg16Sint,
                TextureStorageFormat::Rg16Float => F::Rg16Float,
                TextureStorageFormat::Rgba8Unorm => F::Rgba8Unorm,
                TextureStorageFormat::Rgba8Snorm => F::Rgba8Snorm,
                TextureStorageFormat::Rgba8Uint => F::Rgba8Uint,
                TextureStorageFormat::Rgba8Sint => F::Rgba8Sint,
                TextureStorageFormat::Bgra8Unorm => F::Bgra8Unorm,
                TextureStorageFormat::Rgb10a2Uint => F::Rgb10a2Uint,
                TextureStorageFormat::Rgb10a2Unorm => F::Rgb10a2Unorm,
                TextureStorageFormat::Rg11b10Ufloat => F::Rg11b10Ufloat,
                TextureStorageFormat::R64Uint => F::R64Uint,
                TextureStorageFormat::Rg32Uint => F::Rg32Uint,
                TextureStorageFormat::Rg32Sint => F::Rg32Sint,
                TextureStorageFormat::Rg32Float => F::Rg32Float,
                TextureStorageFormat::Rgba16Uint => F::Rgba16Uint,
                TextureStorageFormat::Rgba16Sint => F::Rgba16Sint,
                TextureStorageFormat::Rgba16Float => F::Rgba16Float,
                TextureStorageFormat::Rgba32Uint => F::Rgba32Uint,
                TextureStorageFormat::Rgba32Sint => F::Rgba32Sint,
                TextureStorageFormat::Rgba32Float => F::Rgba32Float,
                TextureStorageFormat::R16Unorm => F::R16Unorm,
                TextureStorageFormat::R16Snorm => F::R16Snorm,
                TextureStorageFormat::Rg16Unorm => F::Rg16Unorm,
                TextureStorageFormat::Rg16Snorm => F::Rg16Snorm,
                TextureStorageFormat::Rgba16Unorm => F::Rgba16Unorm,
                TextureStorageFormat::Rgba16Snorm => F::Rgba16Snorm,
            }
        }
    }

    impl TextureStorageAccess {
        /// The naga `StorageAccess` this `TextureStorageAccess` stands for.
        fn to_naga(self) -> naga::StorageAccess {
            use naga::StorageAccess as SA;
            let mut out = SA::empty();
            if self.load {
                out |= SA::LOAD;
            }
            if self.store {
                out |= SA::STORE;
            }
            if self.atomic {
                out |= SA::ATOMIC;
            }
            out
        }
    }

    impl TextureClass {
        /// The naga `ImageClass` this `TextureClass` stands for.
        fn to_naga(self) -> naga::ImageClass {
            match self {
                TextureClass::Sampled { kind, multi } => naga::ImageClass::Sampled {
                    kind: kind.to_naga(),
                    multi,
                },
                TextureClass::Storage { format, access } => naga::ImageClass::Storage {
                    format: format.to_naga(),
                    access: access.to_naga(),
                },
                TextureClass::Depth { multi } => naga::ImageClass::Depth { multi },
                TextureClass::External() => naga::ImageClass::External,
            }
        }
    }

    #[test]
    fn texture_dim_round_trip() {
        for d in [
            naga::ImageDimension::D1,
            naga::ImageDimension::D2,
            naga::ImageDimension::D3,
            naga::ImageDimension::Cube,
        ] {
            let ours: TextureDim = TextureDim::from_naga(d);
            let back: naga::ImageDimension = ours.to_naga();
            assert_eq!(d, back);
        }
    }

    #[test]
    fn texture_scalar_kind_round_trip_for_concrete_kinds() {
        for k in [
            naga::ScalarKind::Sint,
            naga::ScalarKind::Uint,
            naga::ScalarKind::Float,
            naga::ScalarKind::Bool,
        ] {
            let ours: TextureScalarKind = TextureScalarKind::from_naga(k);
            let back: naga::ScalarKind = ours.to_naga();
            assert_eq!(k, back);
        }
    }

    #[test]
    fn texture_scalar_kind_abstract_variants_fold_to_float() {
        let ours: TextureScalarKind = TextureScalarKind::from_naga(naga::ScalarKind::AbstractInt);
        assert_eq!(ours, TextureScalarKind::Float);
        let ours: TextureScalarKind = TextureScalarKind::from_naga(naga::ScalarKind::AbstractFloat);
        assert_eq!(ours, TextureScalarKind::Float);
    }

    #[test]
    fn texture_storage_access_round_trip() {
        use naga::StorageAccess as SA;
        for value in [
            SA::empty(),
            SA::LOAD,
            SA::STORE,
            SA::ATOMIC,
            SA::LOAD | SA::STORE,
            SA::LOAD | SA::ATOMIC,
            SA::STORE | SA::ATOMIC,
            SA::LOAD | SA::STORE | SA::ATOMIC,
        ] {
            let ours: TextureStorageAccess = TextureStorageAccess::from_naga(value);
            let back: SA = ours.to_naga();
            assert_eq!(value, back);
        }
    }

    #[test]
    fn texture_class_round_trip_for_each_variant() {
        let cases = [
            naga::ImageClass::Sampled {
                kind: naga::ScalarKind::Float,
                multi: false,
            },
            naga::ImageClass::Sampled {
                kind: naga::ScalarKind::Sint,
                multi: true,
            },
            naga::ImageClass::Storage {
                format: naga::StorageFormat::Rgba8Unorm,
                access: naga::StorageAccess::LOAD | naga::StorageAccess::STORE,
            },
            naga::ImageClass::Depth { multi: false },
            naga::ImageClass::External,
        ];
        for c in cases {
            let ours: TextureClass = TextureClass::from_naga(c);
            let back: naga::ImageClass = ours.to_naga();
            assert_eq!(c, back);
        }
    }
}
