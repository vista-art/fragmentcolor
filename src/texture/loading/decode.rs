//! Decodes an image and picks the format its pixels fill,
//! and checks the pixels a source brings against the format it asks for.

use image::DynamicImage;

use super::TextureSource;
use crate::Size;
use crate::texture::mipmap::read_floats;
use crate::texture::write::bytes_to_u16_vec;
use crate::texture::{TextureError, TextureOptions, check_size, measure_texel};

/// The format picked for a source that stores its own, a mip chain or a KTX2 container,
/// and the call that picked it, which a refusal names.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pick {
    /// The format picked, or `None` to take the source's own.
    pub(crate) format: Option<wgpu::TextureFormat>,
    /// Whether the format is a Material slot's hint rather than the caller's own pick.
    /// A hint applies only where the source's bytes read in it, and never refuses.
    pub(crate) hinted: bool,
    pub(crate) call: &'static str,
}

impl Pick {
    /// What `source` picks, for the call `call`. The default format picks none.
    pub(crate) fn read(source: &TextureSource, call: &'static str) -> Self {
        Self {
            format: pick_override(&source.options),
            hinted: source.hinted,
            call,
        }
    }

    /// The format a source `what`, stored in `built`, uploads as.
    ///
    /// It is the source's own format when nothing was picked, and the pick when the bytes
    /// read the same in it, as in an sRGB twin. A hint that doesn't read falls back to the own
    /// format. Any other pick is `InvalidInput`, which names the call, since the bytes can't be read in it.
    pub(crate) fn apply(
        self,
        built: wgpu::TextureFormat,
        what: &str,
    ) -> Result<wgpu::TextureFormat, TextureError> {
        match self.format {
            None => Ok(built),
            Some(format) if reads_as(built, format) => Ok(format),
            Some(_) if self.hinted => Ok(built),
            Some(format) => Err(TextureError::InvalidInput(format!(
                "{call}: {what} stored in {built:?} can't upload as {format:?}; pass its own format, its sRGB twin, or none",
                call = self.call
            ))),
        }
    }
}

/// Whether bytes laid out in `built` read correctly as `format`, which differs at most in sRGB.
fn reads_as(built: wgpu::TextureFormat, format: wgpu::TextureFormat) -> bool {
    built.remove_srgb_suffix() == format.remove_srgb_suffix()
}

/// The format `options` names, or `None` for the default, which lets the image pick.
pub(in crate::texture) fn pick_override(options: &TextureOptions) -> Option<wgpu::TextureFormat> {
    match options.format {
        crate::TextureFormat::Rgba => None,
        format => Some(format.to_wgpu()),
    }
}

/// Checks that a `size` comes only with raw pixel bytes, given whether the data is
/// `sized`, since a path, a URL, a KTX2 file and a mip chain each carry their own.
pub(in crate::texture) fn check_source(
    sized: bool,
    size: Option<Size>,
) -> Result<(), TextureError> {
    match sized && size.is_some() {
        true => Err(TextureError::InvalidInput(
            "a size describes raw pixel bytes; a path, URL, KTX2 file or Mipmap takes its size from the file".into(),
        )),
        false => Ok(()),
    }
}

/// Decodes an encoded image, its format guessed from the bytes, once each side
/// its header declares fits `limit`, so an image too large to upload is refused
/// before its pixels take any memory. `create_texture` and `Mipmap::build` share it.
pub(in crate::texture) fn decode(bytes: &[u8], limit: u32) -> Result<DynamicImage, TextureError> {
    let (width, height) = read_dimensions(bytes)?;
    check_size(width, height, limit)?;
    Ok(decode_pixels(bytes)?)
}

/// The width and height an encoded image's header declares.
fn read_dimensions(bytes: &[u8]) -> Result<(u32, u32), ImageFailure> {
    let format = image::guess_format(bytes)?;
    let reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    Ok(reader.into_dimensions()?)
}

/// The pixels of an encoded image, its format guessed from the bytes.
pub(in crate::texture) fn decode_pixels(bytes: &[u8]) -> Result<DynamicImage, ImageFailure> {
    let format = image::guess_format(bytes)?;
    let reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    Ok(reader.decode()?)
}

/// A failure of the image library, which `TextureError::MalformedImageError` carries as text.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(crate) struct ImageFailure(#[from] pub(in crate::texture) image::ImageError);

impl From<std::io::Error> for ImageFailure {
    fn from(error: std::io::Error) -> Self {
        ImageFailure(image::ImageError::IoError(error))
    }
}

/// The image library's own text, which already holds the decoder's reason.
impl From<ImageFailure> for TextureError {
    fn from(failure: ImageFailure) -> Self {
        TextureError::MalformedImageError(failure.0.to_string())
    }
}

/// The bytes of the file at `path`. A failed read is an image error, as `image::open` reports it.
pub(in crate::texture) fn read_file(path: &std::path::Path) -> Result<Vec<u8>, ImageFailure> {
    Ok(std::fs::read(path)?)
}

/// Map an `image::ColorType` to a sensible default `wgpu::TextureFormat` for sampling.
pub(crate) fn infer_format(color: image::ColorType) -> wgpu::TextureFormat {
    match color {
        image::ColorType::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        image::ColorType::L8 => wgpu::TextureFormat::R8Unorm,
        image::ColorType::La8 => wgpu::TextureFormat::Rg8Unorm,
        image::ColorType::Rgb8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        image::ColorType::L16 => wgpu::TextureFormat::R16Unorm,
        image::ColorType::La16 => wgpu::TextureFormat::Rg16Unorm,
        image::ColorType::Rgb16 => wgpu::TextureFormat::Rgba16Unorm,
        image::ColorType::Rgba16 => wgpu::TextureFormat::Rgba16Unorm,
        image::ColorType::Rgb32F => wgpu::TextureFormat::Rgba16Float,
        image::ColorType::Rgba32F => wgpu::TextureFormat::Rgba16Float,
        _ => wgpu::TextureFormat::Rgba8UnormSrgb,
    }
}

/// The texel bytes and size of a `uint8` array of `shape` for `format`.
/// Channels that match the format's bytes per pixel pass as they are, and
/// 1 or 3 channels gain an opaque alpha for an 8-bit RGBA or BGRA format.
/// The shape, `(h, w)` or `(h, w, channels)`, is checked before `read` collects the bytes.
#[cfg(any(python, test))]
pub(crate) fn array_pixels(
    shape: &[usize],
    read: impl FnOnce() -> Vec<u8>,
    format: wgpu::TextureFormat,
) -> Result<(Vec<u8>, Size), TextureError> {
    let invalid = || {
        TextureError::InvalidInput(format!(
            "an ndarray of shape {shape:?} doesn't fit a {format:?} texture"
        ))
    };
    let (height, width, channels) = match *shape {
        [height, width] => (height, width, 1),
        [height, width, channels] => (height, width, channels),
        _ => return Err(invalid()),
    };
    let side = |value: usize| u32::try_from(value).ok().ok_or_else(invalid);
    let size = Size::new(side(width)?, side(height)?, None);
    if channels == measure_texel(format)? as usize {
        return Ok((read(), size));
    }
    if !holds_rgba8(format) || !matches!(channels, 1 | 3) {
        return Err(invalid());
    }
    let length = width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(invalid)?;
    let mut pixels = Vec::with_capacity(length);
    for texel in read().chunks_exact(channels) {
        match *texel {
            [value] => pixels.extend_from_slice(&[value, value, value, 255]),
            [red, green, blue] => pixels.extend_from_slice(&[red, green, blue, 255]),
            _ => return Err(invalid()),
        }
    }
    Ok((pixels, size))
}

/// Checks a `size` and `format` for a source with RGBA8 pixels at the size `own`,
/// such as a JavaScript `ImageData` or canvas. A different size, or a
/// format other than 8-bit RGBA or BGRA, would misread the pixels it holds.
#[cfg(any(wasm, test))]
pub(crate) fn check_pixels(
    own: Size,
    size: Option<Size>,
    format: wgpu::TextureFormat,
) -> Result<(), TextureError> {
    let flat = |size: Size| (size.width, size.height, size.depth.unwrap_or(1));
    if let Some(size) = size.filter(|size| flat(*size) != flat(own)) {
        return Err(TextureError::InvalidInput(format!(
            "size {size:?} differs from the source's {}x{}",
            own.width, own.height
        )));
    }
    match holds_rgba8(format) {
        true => Ok(()),
        false => Err(TextureError::InvalidInput(format!(
            "the source holds RGBA8 pixels, which a {format:?} texture can't take"
        ))),
    }
}

/// True for the formats that store 8-bit RGBA or BGRA texels.
#[cfg(any(python, wasm, test))]
fn holds_rgba8(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb
    )
}

/// Swaps red and blue in 4-byte `pixels` when `format` stores BGRA.
/// The swap is its own inverse, so it turns RGBA into BGRA and back.
pub(crate) fn swap_bgra(pixels: &mut [u8], format: wgpu::TextureFormat) {
    if matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel.swap(0, 2);
        }
    }
}

/// The image as 4-byte pixels in the channel order of `format`, blue first for BGRA.
pub(in crate::texture) fn decode_rgba8(
    image: &DynamicImage,
    format: wgpu::TextureFormat,
) -> image::RgbaImage {
    let mut pixels = image.to_rgba8();
    swap_bgra(&mut pixels, format);
    pixels
}

/// Wrap raw pixel bytes (already laid out for `format` at `width x height`)
/// into a `DynamicImage` so [`build_mip_chain`] can downsample them with
/// the same Triangle filter used by the inline path. Mirrors the format
/// dispatch in [`write_levels`] but produces an in-memory image
/// instead of writing to the GPU.
pub(crate) fn bytes_as_image(
    bytes: &[u8],
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> Result<DynamicImage, TextureError> {
    let pixel_count = (width as usize) * (height as usize);
    let wrap_err = |kind: &str| {
        TextureError::CreateTextureError(format!(
            "Failed to wrap raw bytes as {} for prepare_raw",
            kind
        ))
    };
    Ok(match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            // The image holds RGBA, so BGRA bytes swap here and back when the chain is built.
            let mut pixels = bytes[..pixel_count * 4].to_vec();
            swap_bgra(&mut pixels, format);
            DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, pixels)
                    .ok_or_else(|| wrap_err("RgbaImage"))?,
            )
        }
        wgpu::TextureFormat::R8Unorm => DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(width, height, bytes[..pixel_count].to_vec())
                .ok_or_else(|| wrap_err("GrayImage"))?,
        ),
        wgpu::TextureFormat::Rg8Unorm => DynamicImage::ImageLumaA8(
            image::GrayAlphaImage::from_raw(width, height, bytes[..pixel_count * 2].to_vec())
                .ok_or_else(|| wrap_err("GrayAlphaImage"))?,
        ),
        wgpu::TextureFormat::R16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 2]);
            DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("Luma16"))?,
            )
        }
        wgpu::TextureFormat::Rg16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 4]);
            DynamicImage::ImageLumaA16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("LumaA16"))?,
            )
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 8]);
            DynamicImage::ImageRgba16(
                image::ImageBuffer::from_raw(width, height, words)
                    .ok_or_else(|| wrap_err("Rgba16"))?,
            )
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            let length = pixel_count * measure_texel(format)? as usize;
            let values = read_floats(&bytes[..length], format);
            DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, values)
                    .ok_or_else(|| wrap_err("Rgba32F"))?,
            )
        }
        _ => {
            return Err(TextureError::CreateTextureError(format!(
                "bytes_as_image: format {:?} is not supported",
                format
            )));
        }
    })
}
