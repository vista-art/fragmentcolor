//! `TextureFormat`, and what each format lets a device do:
//! the features, usages, uploads and copies it takes.

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use super::{TextureError, supports_cpu_mipmaps};
use crate::RenderContext;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass(from_py_object, eq, hash, frozen))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    R8Unorm,
    Rg8Unorm,
    R16Unorm,
    Rg16Unorm,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Rgba16Unorm,
    Rgba16Float,
    Rgba32Float,
    Rgba32Uint,
    Rgba32Sint,
    R32Uint,
    R32Sint,
    R32Float,
    Depth32Float,
    #[default]
    Rgba,
    Bgra,
    Lab,
    L8,
}

/// Every format in the order `TextureFormat` declares it,
/// so the index of a format is the number JavaScript holds for it.
#[cfg(any(wasm, test))]
pub(crate) const FORMATS: [TextureFormat; 20] = [
    TextureFormat::R8Unorm,
    TextureFormat::Rg8Unorm,
    TextureFormat::R16Unorm,
    TextureFormat::Rg16Unorm,
    TextureFormat::Rgba8Unorm,
    TextureFormat::Rgba8UnormSrgb,
    TextureFormat::Bgra8Unorm,
    TextureFormat::Rgba16Unorm,
    TextureFormat::Rgba16Float,
    TextureFormat::Rgba32Float,
    TextureFormat::Rgba32Uint,
    TextureFormat::Rgba32Sint,
    TextureFormat::R32Uint,
    TextureFormat::R32Sint,
    TextureFormat::R32Float,
    TextureFormat::Depth32Float,
    TextureFormat::Rgba,
    TextureFormat::Bgra,
    TextureFormat::Lab,
    TextureFormat::L8,
];

impl TextureFormat {
    /// The wgpu format this format stands for. The logical names map to a concrete format.
    pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::R16Unorm => wgpu::TextureFormat::R16Unorm,
            TextureFormat::Rg16Unorm => wgpu::TextureFormat::Rg16Unorm,
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            TextureFormat::Rgba16Unorm => wgpu::TextureFormat::Rgba16Unorm,
            TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
            TextureFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
            TextureFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
            TextureFormat::R32Uint => wgpu::TextureFormat::R32Uint,
            TextureFormat::R32Sint => wgpu::TextureFormat::R32Sint,
            TextureFormat::R32Float => wgpu::TextureFormat::R32Float,
            TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
            TextureFormat::Rgba => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra => wgpu::TextureFormat::Bgra8Unorm,
            TextureFormat::Lab => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::L8 => wgpu::TextureFormat::R8Unorm,
        }
    }

    /// The format for a wgpu format, `Rgba8Unorm` for one FragmentColor has no name for.
    pub(crate) fn from_wgpu(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::R8Unorm => TextureFormat::R8Unorm,
            wgpu::TextureFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
            wgpu::TextureFormat::R16Unorm => TextureFormat::R16Unorm,
            wgpu::TextureFormat::Rg16Unorm => TextureFormat::Rg16Unorm,
            wgpu::TextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba16Unorm => TextureFormat::Rgba16Unorm,
            wgpu::TextureFormat::Rgba16Float => TextureFormat::Rgba16Float,
            wgpu::TextureFormat::Rgba32Float => TextureFormat::Rgba32Float,
            wgpu::TextureFormat::Rgba32Uint => TextureFormat::Rgba32Uint,
            wgpu::TextureFormat::Rgba32Sint => TextureFormat::Rgba32Sint,
            wgpu::TextureFormat::R32Uint => TextureFormat::R32Uint,
            wgpu::TextureFormat::R32Sint => TextureFormat::R32Sint,
            wgpu::TextureFormat::R32Float => TextureFormat::R32Float,
            wgpu::TextureFormat::Depth32Float => TextureFormat::Depth32Float,
            _ => TextureFormat::Rgba8Unorm,
        }
    }
}

/// What the device lets a texture of `format` do, as wgpu checks it.
/// A backend below WebGPU, such as GLES or WebGL2, reports its own table,
/// and every other backend takes the WebGPU baseline for the device's features.
/// It reads the context's cached features and downlevel flags, so a check can run it each frame.
pub(crate) fn format_features(
    context: &RenderContext,
    format: wgpu::TextureFormat,
) -> wgpu::TextureFormatFeatures {
    let features = context.features;
    let below = !context
        .downlevel
        .contains(wgpu::DownlevelFlags::WEBGPU_TEXTURE_FORMAT_SUPPORT);
    let reports =
        features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) || below;
    match context.adapter.as_ref().filter(|_| reports) {
        Some(adapter) => adapter.get_texture_format_features(format),
        None => format.guaranteed_format_features(features),
    }
}

/// Checks the device makes `format` with `usage` and `dimension`,
/// so a texture wgpu would refuse never reaches `device.create_texture`,
/// and returns the usage to create it with, as `add_atomic` widens it.
pub(crate) fn check_format(
    context: &RenderContext,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    dimension: wgpu::TextureDimension,
) -> Result<wgpu::TextureUsages, TextureError> {
    check_feature(context.device.features(), format)?;
    let allowed = format_features(context, format).allowed_usages;
    check_support(allowed, format, usage, dimension)?;
    Ok(add_atomic(context.features, allowed, usage))
}

/// `usage` with `STORAGE_ATOMIC` added when it holds `STORAGE_BINDING`
/// and the device binds the format atomically: it has `TEXTURE_ATOMIC`,
/// and the format `allowed` takes the usage. Metal needs it on the texture to run
/// texture atomics, wgpu never checks it at bind time, and no `TextureUsage` flag names it.
fn add_atomic(
    features: wgpu::Features,
    allowed: wgpu::TextureUsages,
    usage: wgpu::TextureUsages,
) -> wgpu::TextureUsages {
    let binds = usage.contains(wgpu::TextureUsages::STORAGE_BINDING)
        && allowed.contains(wgpu::TextureUsages::STORAGE_ATOMIC)
        && features.contains(wgpu::Features::TEXTURE_ATOMIC);
    match binds {
        true => usage | wgpu::TextureUsages::STORAGE_ATOMIC,
        false => usage,
    }
}

/// Checks that the device's `features` hold every feature `format` needs.
pub(crate) fn check_feature(
    features: wgpu::Features,
    format: wgpu::TextureFormat,
) -> Result<(), TextureError> {
    let missing = format.required_features() - features;
    if missing.is_empty() {
        return Ok(());
    }
    let names: Vec<&str> = missing.iter_names().map(|(name, _)| name).collect();
    Err(TextureError::UnsupportedFormatForUsage {
        format: format!("{format:?}"),
        missing_feature: names.join(" | "),
    })
}

/// Checks that a copy can fill a texture of `format` with pixels.
/// wgpu takes no buffer copy into a depth format, so such a texture is drawn into instead.
pub(crate) fn check_upload(format: wgpu::TextureFormat) -> Result<(), TextureError> {
    match format.is_depth_stencil_format() {
        true => Err(TextureError::InvalidInput(format!(
            "{format:?} takes no uploaded pixels, render into it instead"
        ))),
        false => Ok(()),
    }
}

/// Checks that a decoded image converts into `format`,
/// one the CPU mipmap path fills. Integer formats take raw bytes.
pub(crate) fn check_decode(format: wgpu::TextureFormat) -> Result<(), TextureError> {
    match supports_cpu_mipmaps(format) {
        true => Ok(()),
        false => Err(TextureError::InvalidInput(format!(
            "can't decode an image into {format:?}, pass raw bytes with a size"
        ))),
    }
}

/// Checks `usage` against the usages `allowed` for `format`,
/// and that a depth format makes a 2D texture, the only kind it has.
pub(crate) fn check_support(
    allowed: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    dimension: wgpu::TextureDimension,
) -> Result<(), TextureError> {
    let missing = usage - allowed;
    if !missing.is_empty() {
        let names: Vec<&str> = missing.iter_names().map(|(name, _)| name).collect();
        return Err(TextureError::InvalidInput(format!(
            "a {format:?} texture can't take the usage {}",
            names.join(" | ")
        )));
    }
    if format.is_depth_stencil_format() && dimension != wgpu::TextureDimension::D2 {
        return Err(TextureError::InvalidInput(format!(
            "a {format:?} texture has one layer, so its depth must be 1"
        )));
    }
    Ok(())
}

/// The bytes a texel of `format` takes in a copy, one texel at a time.
/// A block-compressed format, and a depth format with no copy size, are refused.
pub(crate) fn measure_texel(format: wgpu::TextureFormat) -> Result<u32, TextureError> {
    match (format.block_dimensions(), format.block_copy_size(None)) {
        ((1, 1), Some(size)) => Ok(size),
        _ => Err(TextureError::InvalidInput(format!(
            "can't copy pixels of {format:?}, write and read take formats with a fixed size per pixel"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: every concrete format comes back from its wgpu format as itself,
    // and each logical name comes back as the concrete format it stands for.
    #[test]
    fn converts_between_public_and_wgpu_formats() {
        let aliases = [
            (TextureFormat::Rgba, TextureFormat::Rgba8Unorm),
            (TextureFormat::Bgra, TextureFormat::Bgra8Unorm),
            (TextureFormat::Lab, TextureFormat::Rg8Unorm),
            (TextureFormat::L8, TextureFormat::R8Unorm),
        ];

        for format in FORMATS {
            let back = TextureFormat::from_wgpu(format.to_wgpu());

            let alias = aliases.iter().find(|(alias, _)| *alias == format);
            match alias {
                Some((_, concrete)) => assert_eq!(back, *concrete, "{format:?}"),
                None => assert_eq!(back, format),
            }
        }
    }

    // Story: JavaScript holds each format as a number, its place in the enum,
    // and `read_format` reads it back through `FORMATS`. The table pins each number,
    // so a variant moved or left out of the list fails here.
    #[test]
    fn formats_keep_their_codes() {
        let pins = [
            (0, TextureFormat::R8Unorm),
            (1, TextureFormat::Rg8Unorm),
            (2, TextureFormat::R16Unorm),
            (3, TextureFormat::Rg16Unorm),
            (4, TextureFormat::Rgba8Unorm),
            (5, TextureFormat::Rgba8UnormSrgb),
            (6, TextureFormat::Bgra8Unorm),
            (7, TextureFormat::Rgba16Unorm),
            (8, TextureFormat::Rgba16Float),
            (9, TextureFormat::Rgba32Float),
            (10, TextureFormat::Rgba32Uint),
            (11, TextureFormat::Rgba32Sint),
            (12, TextureFormat::R32Uint),
            (13, TextureFormat::R32Sint),
            (14, TextureFormat::R32Float),
            (15, TextureFormat::Depth32Float),
            (16, TextureFormat::Rgba),
            (17, TextureFormat::Bgra),
            (18, TextureFormat::Lab),
            (19, TextureFormat::L8),
        ];

        assert_eq!(FORMATS.len(), pins.len());
        for (code, format) in pins {
            assert_eq!(format as usize, code, "{format:?}");
            assert_eq!(FORMATS[code], format);
        }
    }

    // Story: a storage texture takes wgpu's STORAGE_ATOMIC usage only where the device
    // has TEXTURE_ATOMIC and binds the format atomically, as R32Uint but never R32Float.
    #[test]
    fn adds_atomic_usage_where_the_device_binds_it() {
        let atomic = wgpu::Features::TEXTURE_ATOMIC;
        let none = wgpu::Features::empty();
        let allowed = |format: wgpu::TextureFormat, features| {
            format.guaranteed_format_features(features).allowed_usages
        };
        let uint = allowed(wgpu::TextureFormat::R32Uint, atomic);
        let storage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let sampled = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC;

        let added = add_atomic(atomic, uint, storage);

        assert_eq!(added, storage | wgpu::TextureUsages::STORAGE_ATOMIC);
        assert_eq!(add_atomic(none, uint, storage), storage);
        let plain = allowed(wgpu::TextureFormat::R32Uint, none);
        assert_eq!(add_atomic(atomic, plain, storage), storage);
        assert_eq!(add_atomic(atomic, uint, sampled), sampled);
        let float = allowed(wgpu::TextureFormat::R32Float, atomic);
        assert_eq!(add_atomic(atomic, float, storage), storage);
    }

    // Story: Default logical formats map to practical wgpu defaults.
    #[test]
    fn maps_logical_defaults_to_wgpu() {
        // Arrange / Act
        let rgba: wgpu::TextureFormat = TextureFormat::Rgba.to_wgpu();
        let bgra: wgpu::TextureFormat = TextureFormat::Bgra.to_wgpu();

        // Assert
        assert_eq!(rgba, wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(bgra, wgpu::TextureFormat::Bgra8Unorm);
    }
}
