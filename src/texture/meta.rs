//! Public, uniffi-bindable mirrors of the `naga` image/sampler metadata that
//! are needed by the runtime to construct wgpu bind-group layouts for textures.
//!
//! These types live alongside `TextureMeta` so that downstream `UniformData`
//! variants (and eventually `UniformData` itself) can be exposed across the
//! uniffi (mobile) FFI without taking a dependency on `naga` types.

#[cfg(python)]
use pyo3::prelude::*;

/// Image-view dimension. Mirrors `naga::ImageDimension`.
#[cfg_attr(python, pyclass)]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDim {
    D1,
    D2,
    D3,
    Cube,
}

impl From<naga::ImageDimension> for TextureDim {
    fn from(value: naga::ImageDimension) -> Self {
        match value {
            naga::ImageDimension::D1 => TextureDim::D1,
            naga::ImageDimension::D2 => TextureDim::D2,
            naga::ImageDimension::D3 => TextureDim::D3,
            naga::ImageDimension::Cube => TextureDim::Cube,
        }
    }
}

impl From<TextureDim> for naga::ImageDimension {
    fn from(value: TextureDim) -> Self {
        match value {
            TextureDim::D1 => naga::ImageDimension::D1,
            TextureDim::D2 => naga::ImageDimension::D2,
            TextureDim::D3 => naga::ImageDimension::D3,
            TextureDim::Cube => naga::ImageDimension::Cube,
        }
    }
}

/// Texel scalar kind for sampled textures. Mirrors the relevant `naga::ScalarKind`
/// variants. Naga's abstract numeric variants are folded into `Float` since they
/// cannot reach the runtime backends.
#[cfg_attr(python, pyclass)]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureScalarKind {
    Sint,
    Uint,
    Float,
    Bool,
}

impl From<naga::ScalarKind> for TextureScalarKind {
    fn from(value: naga::ScalarKind) -> Self {
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

impl From<TextureScalarKind> for naga::ScalarKind {
    fn from(value: TextureScalarKind) -> Self {
        match value {
            TextureScalarKind::Sint => naga::ScalarKind::Sint,
            TextureScalarKind::Uint => naga::ScalarKind::Uint,
            TextureScalarKind::Float => naga::ScalarKind::Float,
            TextureScalarKind::Bool => naga::ScalarKind::Bool,
        }
    }
}

/// Storage-texture format. Mirrors `naga::StorageFormat`.
///
/// Variants outside of the set the renderer translates to `wgpu::TextureFormat`
/// fall through to the renderer's `Rgba8Unorm` fallback.
#[cfg_attr(python, pyclass)]
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

impl From<naga::StorageFormat> for TextureStorageFormat {
    fn from(value: naga::StorageFormat) -> Self {
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

impl From<TextureStorageFormat> for naga::StorageFormat {
    fn from(value: TextureStorageFormat) -> Self {
        use naga::StorageFormat as F;
        match value {
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
#[cfg_attr(python, pyclass)]
#[cfg_attr(mobile, derive(uniffi::Record))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureStorageAccess {
    pub load: bool,
    pub store: bool,
    pub atomic: bool,
}

impl From<naga::StorageAccess> for TextureStorageAccess {
    fn from(value: naga::StorageAccess) -> Self {
        use naga::StorageAccess as SA;
        TextureStorageAccess {
            load: value.contains(SA::LOAD),
            store: value.contains(SA::STORE),
            atomic: value.contains(SA::ATOMIC),
        }
    }
}

impl From<TextureStorageAccess> for naga::StorageAccess {
    fn from(value: TextureStorageAccess) -> Self {
        use naga::StorageAccess as SA;
        let mut out = SA::empty();
        if value.load {
            out |= SA::LOAD;
        }
        if value.store {
            out |= SA::STORE;
        }
        if value.atomic {
            out |= SA::ATOMIC;
        }
        out
    }
}

/// Image sub-class. Mirrors `naga::ImageClass`.
#[cfg_attr(python, pyclass)]
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

impl From<naga::ImageClass> for TextureClass {
    fn from(value: naga::ImageClass) -> Self {
        match value {
            naga::ImageClass::Sampled { kind, multi } => TextureClass::Sampled {
                kind: kind.into(),
                multi,
            },
            naga::ImageClass::Storage { format, access } => TextureClass::Storage {
                format: format.into(),
                access: access.into(),
            },
            naga::ImageClass::Depth { multi } => TextureClass::Depth { multi },
            naga::ImageClass::External => TextureClass::External(),
        }
    }
}

impl From<TextureClass> for naga::ImageClass {
    fn from(value: TextureClass) -> Self {
        match value {
            TextureClass::Sampled { kind, multi } => naga::ImageClass::Sampled {
                kind: kind.into(),
                multi,
            },
            TextureClass::Storage { format, access } => naga::ImageClass::Storage {
                format: format.into(),
                access: access.into(),
            },
            TextureClass::Depth { multi } => naga::ImageClass::Depth { multi },
            TextureClass::External() => naga::ImageClass::External,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_dim_round_trip() {
        for d in [
            naga::ImageDimension::D1,
            naga::ImageDimension::D2,
            naga::ImageDimension::D3,
            naga::ImageDimension::Cube,
        ] {
            let ours: TextureDim = d.into();
            let back: naga::ImageDimension = ours.into();
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
            let ours: TextureScalarKind = k.into();
            let back: naga::ScalarKind = ours.into();
            assert_eq!(k, back);
        }
    }

    #[test]
    fn texture_scalar_kind_abstract_variants_fold_to_float() {
        let ours: TextureScalarKind = naga::ScalarKind::AbstractInt.into();
        assert_eq!(ours, TextureScalarKind::Float);
        let ours: TextureScalarKind = naga::ScalarKind::AbstractFloat.into();
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
            let ours: TextureStorageAccess = value.into();
            let back: SA = ours.into();
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
            let ours: TextureClass = c.into();
            let back: naga::ImageClass = ours.into();
            assert_eq!(c, back);
        }
    }
}
