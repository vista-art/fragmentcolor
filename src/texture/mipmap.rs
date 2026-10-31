//! `Mipmap`, a mip chain built on the CPU,
//! and the downsampling that builds its levels.

use image::{DynamicImage, GenericImageView};
use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use super::loading::decode::{bytes_as_image, check_source, decode, decode_rgba8, read_file};
use super::{TextureData, TextureError, TextureFormat, TextureInput, measure_texel};
use crate::Size;

/// A pre-computed CPU mipmap chain. Build with [`Mipmap::build`], then pass
/// to [`crate::Renderer::create_texture`] (the unified entry point
/// dispatches via `From<Mipmap> for TextureInput`).
///
/// Levels are tightly-packed bytes, level 0 first, with the format's bytes per pixel
/// times `max(1, base_w >> level) * max(1, base_h >> level)` bytes per level.
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/texture/mipmap/mipmap.md")]
pub struct Mipmap {
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) base_size: (u32, u32),
    /// Tightly-packed mip levels, level 0 first. Wrapped in `Arc` so cloning
    /// the handle (or its enclosing `TextureInput::Prepared` variant) does
    /// not duplicate the (potentially large) byte buffers.
    pub(crate) levels: std::sync::Arc<Vec<Vec<u8>>>,
}

crate::impl_fc_kind!(Mipmap, "Mipmap");
// Brand-anchored TryFrom<&JsValue> so the JS dispatch in TextureInput's
// TryFrom can detect a Mipmap handle (recovered via __wbg_ptr +
// __fc_kind brand) and route it as TextureInput::Prepared.
crate::impl_js_bridge!(Mipmap, crate::texture::TextureError);

/// The side limit a chain decodes with. It has no device yet,
/// and `create_texture` checks its size against the device when it uploads.
const ANY_SIDE: u32 = u32::MAX;

impl Mipmap {
    /// Build a mip chain from a [`TextureInput`]. Pure CPU work — call from
    /// any thread (worker, thread pool, async task), then hand the chain to
    /// [`crate::Renderer::create_texture`] on the renderer thread for a
    /// GPU-only upload.
    ///
    /// Same `TextureInput` transport as `Renderer::create_texture` and
    /// `Renderer::create_storage_texture` — one vocabulary across the API.
    /// The `From<T>` impls cover the common call shapes:
    ///
    /// ```ignore
    /// // Encoded image bytes (PNG / JPEG / etc.) — size inferred from the image.
    /// Mipmap::build((png_bytes, TextureFormat::Rgba8UnormSrgb))?;
    ///
    /// // Raw pixel bytes — caller declares the dimensions.
    /// Mipmap::build((rgba_bytes, TextureFormat::Rgba8UnormSrgb, [w, h]))?;
    /// ```
    ///
    /// `build` requires a sync-friendly `data` variant: `Bytes` or
    /// `Path` (file IO). `Url` (needs async fetch),
    /// `Ktx2*` (already pre-baked), `Prepared` (already a chain),
    /// `CloneOf` (existing texture), and `Empty` (nothing to chain) all
    /// return [`TextureError::InvalidInput`].
    ///
    /// Supported formats for the mipmap chain: `Rgba8Unorm`,
    /// `Rgba8UnormSrgb`, `Bgra8Unorm`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`,
    /// `Rg16Unorm`, `Rgba16Unorm`, `Rgba16Float`, `Rgba32Float`, and the
    /// aliases for them. Other formats return [`TextureError::UnsupportedMipmapFormat`].
    #[lsp_doc("docs/api/texture/mipmap/build.md")]
    pub fn build(input: impl Into<TextureInput>) -> Result<Mipmap, TextureError> {
        Self::build_chain(input.into())
    }

    /// The chain `build` makes, from a source the CPU reads at once.
    fn build_chain(input: TextureInput) -> Result<Mipmap, TextureError> {
        let TextureInput { data, options } = input;
        check_source(data.is_sized(), options.size)?;
        let public_format = options.format;
        let format: wgpu::TextureFormat = public_format.to_wgpu();
        if !supports_cpu_mipmaps(format) {
            return Err(TextureError::UnsupportedMipmapFormat {
                format: public_format,
            });
        }
        // Raw pixels keep their bytes as level 0, and only the levels below it are resampled.
        let (image, raw) = match data {
            // Encoded bytes path. If `options.size` is present, treat as raw
            // pixel data; otherwise decode internally.
            TextureData::Bytes(bytes) => match options.size {
                None => (decode(&bytes, ANY_SIDE)?, None),
                Some(size) => {
                    let extent: wgpu::Extent3d = size.to_extent();
                    if extent.width == 0 || extent.height == 0 {
                        return Err(TextureError::InvalidInput(
                            "build: size must be non-zero in both dimensions".into(),
                        ));
                    }
                    if extent.depth_or_array_layers != 1 {
                        return Err(TextureError::InvalidInput(format!(
                            "build: a mip chain resamples one layer, got a depth of {}",
                            extent.depth_or_array_layers
                        )));
                    }
                    let texel = measure_texel(format)? as usize;
                    let expected = texel * (extent.width as usize) * (extent.height as usize);
                    if bytes.len() != expected {
                        return Err(TextureError::InvalidInput(format!(
                            "build: expected {} bytes for {}x{} @ {} bpp, got {}",
                            expected,
                            extent.width,
                            extent.height,
                            texel,
                            bytes.len()
                        )));
                    }
                    let image = bytes_as_image(&bytes, extent.width, extent.height, format)?;
                    (image, Some(bytes))
                }
            },
            TextureData::Path(path) => (decode(&read_file(&path)?, ANY_SIDE)?, None),
            // Async / pre-baked / non-prep variants don't fit the sync CPU
            // mipmap pipeline. Surface a clear typed error so callers can
            // match on the same `InvalidInput` variant they already match
            // on for shape mismatches.
            TextureData::Empty => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Empty has no bytes to chain".into(),
                ));
            }
            TextureData::Url(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Url requires async fetch — fetch bytes first, then call build with the bytes".into(),
                ));
            }
            TextureData::Ktx2Bytes(_) | TextureData::Ktx2Path(_) | TextureData::Ktx2Url(_) => {
                return Err(TextureError::InvalidInput(
                    "build: KTX2 inputs already carry a pre-baked mip chain — pass them directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::Prepared(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::Prepared is already a Mipmap — pass it directly to Renderer::create_texture".into(),
                ));
            }
            TextureData::CloneOf(_) => {
                return Err(TextureError::InvalidInput(
                    "build: TextureData::CloneOf wraps an existing texture; nothing to chain"
                        .into(),
                ));
            }
        };
        Self::from_dynamic_image(&image, public_format, format, raw)
    }

    /// The chain of `image`, with `raw`, the caller's own bytes for it, as level 0.
    fn from_dynamic_image(
        image: &DynamicImage,
        public_format: crate::TextureFormat,
        format: wgpu::TextureFormat,
        raw: Option<Vec<u8>>,
    ) -> Result<Mipmap, TextureError> {
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidInput(
                "build: decoded image has zero width or height".into(),
            ));
        }
        let mut levels = build_mip_chain(image, format).map_err(|err| match err {
            // Surface the format mismatch as the typed variant so callers
            // matching on UnsupportedMipmapFormat catch it both at the
            // top-level guard and in the inner dispatcher.
            TextureError::CreateTextureError(_) => TextureError::UnsupportedMipmapFormat {
                format: public_format,
            },
            other => other,
        })?;
        if let (Some(raw), Some(top)) = (raw, levels.first_mut()) {
            *top = raw;
        }
        Ok(Mipmap {
            format,
            base_size: (width, height),
            levels: std::sync::Arc::new(levels),
        })
    }
}

impl Mipmap {
    #[lsp_doc("docs/api/texture/mipmap/format.md")]
    pub fn format(&self) -> crate::TextureFormat {
        TextureFormat::from_wgpu(self.format)
    }

    #[lsp_doc("docs/api/texture/mipmap/size.md")]
    pub fn size(&self) -> Size {
        Size::from(self.base_size)
    }

    #[lsp_doc("docs/api/texture/mipmap/levels.md")]
    pub fn levels(&self) -> &[Vec<u8>] {
        &self.levels
    }

    #[lsp_doc("docs/api/texture/mipmap/level.md")]
    pub fn level(&self, index: usize) -> Result<&[u8], TextureError> {
        self.levels.get(index).map(Vec::as_slice).ok_or_else(|| {
            TextureError::InvalidInput(format!(
                "level {index} out of range; the chain has {} levels",
                self.levels.len()
            ))
        })
    }

    #[lsp_doc("docs/api/texture/mipmap/count.md")]
    pub fn count(&self) -> usize {
        self.levels.len()
    }
}

/// Number of mip levels in a full chain that goes down to 1×1 for the given dimensions.
/// Returns 1 for any zero or 1×1 input.
pub(crate) fn mip_level_count_for(width: u32, height: u32) -> u32 {
    let max_dim = width.max(height);
    if max_dim <= 1 {
        return 1;
    }
    1 + max_dim.ilog2()
}

/// True for formats the CPU mipmap dispatcher in [`write_image_levels`] can fill.
/// Covers every wgpu format we know how to decode a `DynamicImage` into via the
/// `image` crate's `to_*` helpers, the float formats through `to_rgba32f`.
pub(crate) fn supports_cpu_mipmaps(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
            | wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::Rg8Unorm
            | wgpu::TextureFormat::R16Unorm
            | wgpu::TextureFormat::Rg16Unorm
            | wgpu::TextureFormat::Rgba16Unorm
            | wgpu::TextureFormat::Rgba16Float
            | wgpu::TextureFormat::Rgba32Float
    )
}

/// The bytes of f32 `values` in `format`: half floats saturated to their range
/// for `Rgba16Float`, and the values exactly as they are for `Rgba32Float`.
pub(super) fn pack_floats(values: &[f32], format: wgpu::TextureFormat) -> Vec<u8> {
    match format {
        wgpu::TextureFormat::Rgba16Float => values
            .iter()
            .flat_map(|value| to_half(*value).to_le_bytes())
            .collect(),
        _ => bytemuck::cast_slice(values).to_vec(),
    }
}

/// The f32 values of tightly packed `Rgba16Float` or `Rgba32Float` bytes.
pub(super) fn read_floats(bytes: &[u8], format: wgpu::TextureFormat) -> Vec<f32> {
    match format {
        wgpu::TextureFormat::Rgba16Float => {
            let half = |word: &[u8; 2]| half::f16::from_le_bytes(*word).to_f32();
            bytes.as_chunks::<2>().0.iter().map(half).collect()
        }
        _ => {
            let float = |word: &[u8; 4]| f32::from_le_bytes(*word);
            bytes.as_chunks::<4>().0.iter().map(float).collect()
        }
    }
}

/// `base` and the float levels below it, `count` in all, each half the size
/// of the one before, never smaller than 1x1, and averaged with [`halve`].
pub(super) fn downsample_floats(
    base: image::Rgba32FImage,
    count: usize,
) -> impl Iterator<Item = image::Rgba32FImage> {
    std::iter::successors(Some(base), |level| Some(halve(level))).take(count)
}

/// `image` at half its size with a box filter, so a value past 0 to 1 keeps its size,
/// where the `image` crate's resize clamps a float channel to that range.
/// On an odd side the last texel joins the span before it, so every texel counts once.
pub(super) fn halve(image: &image::Rgba32FImage) -> image::Rgba32FImage {
    let (width, height) = image.dimensions();
    let (half_width, half_height) = ((width / 2).max(1), (height / 2).max(1));
    // The source texels output texel `index` of `half` covers, out of `total`.
    let span = |index: u32, half: u32, total: u32| {
        let edge = |index: u32| (u64::from(index) * u64::from(total) / u64::from(half)) as u32;
        edge(index)..edge(index + 1)
    };
    image::Rgba32FImage::from_fn(half_width, half_height, |x, y| {
        let (mut sum, mut count) = ([0f64; 4], 0f64);
        for row in span(y, half_height, height) {
            for column in span(x, half_width, width) {
                let texel = image.get_pixel(column, row).0;
                for (total, value) in sum.iter_mut().zip(texel) {
                    *total += f64::from(value);
                }
                count += 1.0;
            }
        }
        image::Rgba(sum.map(|total| (total / count) as f32))
    })
}

/// `value` as a half float, saturated to the half float range,
/// so a value past 65504 stays 65504 where a plain conversion gives infinity.
fn to_half(value: f32) -> half::f16 {
    let limit = half::f16::MAX.to_f32();
    half::f16::from_f32(value.clamp(-limit, limit))
}

/// Pure-CPU mip chain builder used by [`Mipmap::build`]. Mirrors the format
/// dispatch in [`write_image_levels`] but buffers each level's bytes into a
/// `Vec<u8>` instead of writing to a wgpu texture. Caller must have validated
/// that `format` is in [`supports_cpu_mipmaps`].
pub(crate) fn build_mip_chain(
    image: &DynamicImage,
    format: wgpu::TextureFormat,
) -> Result<Vec<Vec<u8>>, TextureError> {
    let (w, h) = image.dimensions();
    let level_count = mip_level_count_for(w, h) as usize;
    let chain = match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            downsample_levels_to_bytes(decode_rgba8(image, format), level_count)
        }
        wgpu::TextureFormat::R8Unorm => downsample_levels_to_bytes(image.to_luma8(), level_count),
        wgpu::TextureFormat::Rg8Unorm => {
            downsample_levels_to_bytes(image.to_luma_alpha8(), level_count)
        }
        wgpu::TextureFormat::R16Unorm => downsample_levels_to_bytes(image.to_luma16(), level_count),
        wgpu::TextureFormat::Rg16Unorm => {
            downsample_levels_to_bytes(image.to_luma_alpha16(), level_count)
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            downsample_levels_to_bytes(image.to_rgba16(), level_count)
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            downsample_floats(image.to_rgba32f(), level_count)
                .map(|level| pack_floats(level.as_raw(), format))
                .collect()
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "build_mip_chain: format {:?} is not supported",
                format
            )));
        }
    };
    Ok(chain)
}

/// Generic level-buffering: starting from `base`, push the bytes of each level
/// into a fresh `Vec<u8>`, then resample to half size with the Triangle filter.
/// Identical resampling to [`write_pixel_levels`] — same Triangle filter, same
/// floor-to-1 dimension shrink — so chains built here render identically to
/// the inline path.
fn downsample_levels_to_bytes<P>(
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    level_count: usize,
) -> Vec<Vec<u8>>
where
    P: image::Pixel + 'static,
    P::Subpixel: bytemuck::NoUninit + 'static,
{
    downsample(base, level_count)
        .map(|level| bytemuck::cast_slice(level.as_raw()).to_vec())
        .collect()
}

/// `base` and the levels below it, `count` in all, each half the size of the
/// one before with the Triangle filter, and never smaller than 1x1.
/// The filter clamps each channel to its type's range, which suits the integer
/// formats, so the float formats go through [`downsample_floats`] instead.
pub(super) fn downsample<P>(
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    count: usize,
) -> impl Iterator<Item = image::ImageBuffer<P, Vec<P::Subpixel>>>
where
    P: image::Pixel + 'static,
    P::Subpixel: 'static,
{
    let half = |level: &image::ImageBuffer<P, Vec<P::Subpixel>>| {
        let (width, height) = ((level.width() / 2).max(1), (level.height() / 2).max(1));
        let filter = image::imageops::FilterType::Triangle;
        Some(image::imageops::resize(level, width, height, filter))
    };
    std::iter::successors(Some(base), half).take(count)
}
