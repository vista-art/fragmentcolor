//! Writes bytes into a texture, a region of it or each of its mip levels.

use std::borrow::Cow;

use image::DynamicImage;

use crate::RenderContext;
use crate::texture::loading::decode::decode_rgba8;
use crate::texture::mipmap::{downsample, downsample_floats, pack_floats, read_floats};
use crate::texture::{Texture, TextureError, TextureRegion, check_decode, measure_texel};

/// Writes `data` into `region` of the texture, or into all of it without a region.
pub(super) fn write(
    tex: &Texture,
    data: &[u8],
    region: Option<TextureRegion>,
) -> Result<(), TextureError> {
    // One snapshot of the texture drives the checks and the write, even across a resize.
    let object = tex.object();
    // Validate usage supports COPY_DST
    if !object.usage.contains(wgpu::TextureUsages::COPY_DST) {
        return Err(TextureError::Error(
            "Texture usage does not allow writes (missing COPY_DST)".into(),
        ));
    }
    super::check_upload(object.format)?;

    let size = object.size;
    let format = object.format;
    let bpp = super::measure_texel(format)?;

    let whole = [size.width, size.height, size.depth_or_array_layers];
    let region = match region {
        None => TextureRegion::new([0; 3], whole),
        Some(region) if region.size.contains(&0) => {
            let [width, height, depth] = region.size;
            return Err(TextureError::InvalidInput(format!(
                "a region needs a width, height and depth of at least 1, got {width}x{height}x{depth}"
            )));
        }
        Some(region) => region,
    };

    // Bounds check: origin and region must lie within texture extent
    if region.origin[0] >= size.width
        || region.origin[1] >= size.height
        || region.origin[2] >= size.depth_or_array_layers
    {
        return Err(TextureError::Error("Origin out of texture bounds".into()));
    }
    let end_x = region.origin[0].saturating_add(region.size[0]);
    let end_y = region.origin[1].saturating_add(region.size[1]);
    let end_z = region.origin[2].saturating_add(region.size[2]);
    if end_x > size.width || end_y > size.height || end_z > size.depth_or_array_layers {
        return Err(TextureError::Error(
            "Write region exceeds texture bounds".into(),
        ));
    }

    // Compute or validate layout
    let rows_per_image = region.rows_per_image.unwrap_or(region.size[1]);
    if let Some(rpi) = region.rows_per_image
        && rpi < region.size[1]
    {
        return Err(TextureError::Error(
            "rows_per_image must be greater than or equal to region height".into(),
        ));
    }
    let row_stride = region.size[0].saturating_mul(bpp);
    let bytes_per_row = match region.bytes_per_row {
        Some(bpr) => {
            if bpr < row_stride {
                return Err(TextureError::Error(
                    "bytes_per_row smaller than row stride".into(),
                ));
            }
            bpr
        }
        None => default_stride(data.len(), row_stride, rows_per_image, region.size)?,
    };

    let required = count_bytes(bytes_per_row, row_stride, rows_per_image, region.size);
    if (data.len() as u64) < required {
        return Err(TextureError::Error(
            "Input data smaller than required for layout".into(),
        ));
    }

    // A mip chain follows a whole new image, and a partial write would leave it stale.
    let levels = object.inner.mip_level_count();
    if levels > 1 {
        let whole = region.origin == [0; 3] && region.size == [size.width, size.height, 1];
        if !whole {
            return Err(TextureError::InvalidInput(
                "the texture has a mip chain, which a partial write would leave stale; write the whole texture, or make it with one level, as mipmaps: false does".into(),
            ));
        }
        if !super::supports_cpu_mipmaps(format) {
            return Err(TextureError::InvalidInput(format!(
                "a write can't rebuild the mip chain of a {format:?} texture"
            )));
        }
        let rows = trim_rows(data, bytes_per_row, row_stride, region.size[1]);
        return write_levels(&tex.context, &rows, &object.inner, size, format, levels);
    }

    // Write
    tex.context.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &object.inner,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: region.origin[0],
                y: region.origin[1],
                z: region.origin[2],
            },
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(rows_per_image),
        },
        wgpu::Extent3d {
            width: region.size[0],
            height: region.size[1],
            depth_or_array_layers: region.size[2],
        },
    );

    Ok(())
}

/// The `height` rows of `data` at `bytes_per_row`, each cut to its `row_stride` bytes.
fn trim_rows(data: &[u8], bytes_per_row: u32, row_stride: u32, height: u32) -> Cow<'_, [u8]> {
    let (stride, used) = (bytes_per_row as usize, row_stride as usize);
    let length = used * height as usize;
    if stride == used {
        return Cow::Borrowed(&data[..length]);
    }
    let rows = data.chunks(stride).take(height as usize);
    Cow::Owned(rows.flat_map(|row| &row[..used]).copied().collect())
}

/// The fewest bytes WebGPU reads for a region of `size`:
/// every row but the last at `bytes_per_row`, then the last row's `row_stride`.
fn count_bytes(bytes_per_row: u32, row_stride: u32, rows_per_image: u32, size: [u32; 3]) -> u64 {
    let rows = (size[2].saturating_sub(1) as u64)
        .saturating_mul(rows_per_image as u64)
        .saturating_add(size[1].saturating_sub(1) as u64);
    (bytes_per_row as u64)
        .saturating_mul(rows)
        .saturating_add(row_stride as u64)
}

/// Row stride for data of `length` bytes passed without `bytes_per_row`.
/// Rows are tightly packed, since `Queue::write_texture` takes any stride,
/// and data sized exactly for rows padded to 256 bytes reads at the padded stride.
/// Any other length is an error, so a buffer for a different region never uploads cropped.
fn default_stride(
    length: usize,
    row_stride: u32,
    rows_per_image: u32,
    size: [u32; 3],
) -> Result<u32, TextureError> {
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = row_stride.div_ceil(align).saturating_mul(align);
    let full = |stride: u32| {
        (stride as u64)
            .saturating_mul(rows_per_image as u64)
            .saturating_mul(size[2] as u64)
    };
    let length = length as u64;
    if length == full(row_stride)
        || length == count_bytes(row_stride, row_stride, rows_per_image, size)
    {
        return Ok(row_stride);
    }
    if length == full(padded) {
        return Ok(padded);
    }
    let [width, height, depth] = size;
    Err(TextureError::InvalidInput(format!(
        "{length} bytes don't fit a {width}x{height}x{depth} region, which takes {} in tight rows or {} in rows padded to {align} bytes; give the region a bytes per row for any other layout",
        full(row_stride),
        full(padded)
    )))
}

/// Uploads `image` with its mips, decoded into the pixels `format` holds.
///
/// A format [`check_decode`] refuses is an error, as it is in `from_image`.
/// Each branch decodes it into the channels and bytes per pixel of `format`,
/// so a 16-bit grayscale PNG keeps its precision as `R16Unorm`, never `to_rgba8`.
/// A unorm format takes its mips from `image::imageops::resize` with the Triangle filter.
/// The float formats take [`downsample_floats`] instead, since that resize clamps them to 0..1.
pub(crate) fn write_image_levels(
    context: &RenderContext,
    image: &DynamicImage,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> Result<(), TextureError> {
    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            write_pixel_levels(
                context,
                decode_rgba8(image, format),
                texture,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::R8Unorm => {
            write_pixel_levels(context, image.to_luma8(), texture, mip_level_count, 1);
        }
        wgpu::TextureFormat::Rg8Unorm => {
            write_pixel_levels(context, image.to_luma_alpha8(), texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::R16Unorm => {
            write_pixel_levels(context, image.to_luma16(), texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::Rg16Unorm => {
            write_pixel_levels(
                context,
                image.to_luma_alpha16(),
                texture,
                mip_level_count,
                4,
            );
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            write_pixel_levels(context, image.to_rgba16(), texture, mip_level_count, 8);
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            write_floats(
                context,
                image.to_rgba32f(),
                texture,
                mip_level_count,
                format,
                None,
            )?;
        }
        _ => check_decode(format)?,
    }
    Ok(())
}

/// Uploads `base` and the float levels below it, `count` in all, packed for `format`.
/// `raw`, the caller's own bytes for `base`, uploads as level 0 byte for byte.
fn write_floats(
    context: &RenderContext,
    base: image::Rgba32FImage,
    texture: &wgpu::Texture,
    count: u32,
    format: wgpu::TextureFormat,
    raw: Option<&[u8]>,
) -> Result<(), TextureError> {
    let size = measure_texel(format)?;
    for (level, image) in downsample_floats(base, count as usize).enumerate() {
        let bytes = match (level, raw) {
            (0, Some(raw)) => std::borrow::Cow::Borrowed(raw),
            _ => std::borrow::Cow::Owned(pack_floats(image.as_raw(), format)),
        };
        let level = level as u32;
        write_level(context, texture, level, &bytes, image.dimensions(), size);
    }
    Ok(())
}

/// Wrap pre-decoded raw bytes in the right `ImageBuffer` for `format`, then upload
/// (and downsample for higher mip levels). Mirrors [`write_image_levels`] but starts
/// from a `&[u8]` buffer the caller has already laid out tightly. For 16-bit formats,
/// the byte slice is converted to `Vec<u16>` via `from_le_bytes` (alignment-safe and
/// matches WebGPU's little-endian element ordering).
pub(crate) fn write_levels(
    context: &RenderContext,
    bytes: &[u8],
    texture: &wgpu::Texture,
    full_size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> Result<(), TextureError> {
    if mip_level_count == 1 {
        let bpp = measure_texel(format)?;
        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpp * full_size.width),
                rows_per_image: Some(full_size.height),
            },
            full_size,
        );
        return Ok(());
    }

    // Multi-level: wrap into the right ImageBuffer so write_pixel_levels can resample.
    let w = full_size.width;
    let h = full_size.height;
    let pixel_count = (w as usize) * (h as usize);

    let wrap_err = |kind: &str| {
        TextureError::CreateTextureError(format!(
            "Failed to wrap raw bytes as {} for mipmap generation",
            kind
        ))
    };

    match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => {
            let base = image::RgbaImage::from_raw(w, h, bytes[..pixel_count * 4].to_vec())
                .ok_or_else(|| wrap_err("RgbaImage"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 4);
        }
        wgpu::TextureFormat::R8Unorm => {
            let base: image::GrayImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count].to_vec())
                    .ok_or_else(|| wrap_err("GrayImage"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 1);
        }
        wgpu::TextureFormat::Rg8Unorm => {
            let base: image::GrayAlphaImage =
                image::ImageBuffer::from_raw(w, h, bytes[..pixel_count * 2].to_vec())
                    .ok_or_else(|| wrap_err("GrayAlphaImage"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::R16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 2]);
            let base: image::ImageBuffer<image::Luma<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Luma16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 2);
        }
        wgpu::TextureFormat::Rg16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 4]);
            let base: image::ImageBuffer<image::LumaA<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("LumaA16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 4);
        }
        wgpu::TextureFormat::Rgba16Unorm => {
            let words = bytes_to_u16_vec(&bytes[..pixel_count * 8]);
            let base: image::ImageBuffer<image::Rgba<u16>, Vec<u16>> =
                image::ImageBuffer::from_raw(w, h, words).ok_or_else(|| wrap_err("Rgba16"))?;
            write_pixel_levels(context, base, texture, mip_level_count, 8);
        }
        wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgba32Float => {
            let length = pixel_count * measure_texel(format)? as usize;
            let raw = &bytes[..length];
            let base = image::Rgba32FImage::from_raw(w, h, read_floats(raw, format))
                .ok_or_else(|| wrap_err("Rgba32F"))?;
            write_floats(context, base, texture, mip_level_count, format, Some(raw))?;
        }
        _ => {
            // Caller should have caught this via supports_cpu_mipmaps;
            // be defensive and write a single level.
            let bpp = measure_texel(format)?;
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpp * full_size.width),
                    rows_per_image: Some(full_size.height),
                },
                full_size,
            );
        }
    }
    Ok(())
}

/// Convert a tightly-packed byte slice into `Vec<u16>` by reading each pair of
/// bytes as a little-endian u16. Alignment-safe (the source `&[u8]` need not be
/// u16-aligned) and matches WebGPU's element ordering for 16-bit formats.
pub(super) fn bytes_to_u16_vec(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect()
}

/// Generic worker: upload `base` as mip 0, then iteratively downsample with the
/// Triangle filter until the chain is full. Works for any pixel type the `image`
/// crate's `imageops::resize` supports (Rgba8, Luma8, LumaA8, Luma16, LumaA16,
/// Rgba16). Pixel data is reinterpreted to bytes via `bytemuck::cast_slice`,
/// which is a no-op for `u8` subpixels and a little-endian byte view for wider ones.
/// All FragmentColor target platforms are little-endian, which matches WebGPU's
/// expected byte order.
pub(crate) fn write_pixel_levels<P>(
    context: &RenderContext,
    base: image::ImageBuffer<P, Vec<P::Subpixel>>,
    texture: &wgpu::Texture,
    mip_level_count: u32,
    texel: u32,
) where
    P: image::Pixel + 'static,
    P::Subpixel: bytemuck::NoUninit + 'static,
{
    for (level, image) in downsample(base, mip_level_count as usize).enumerate() {
        let bytes: &[u8] = bytemuck::cast_slice(image.as_raw());
        write_level(
            context,
            texture,
            level as u32,
            bytes,
            image.dimensions(),
            texel,
        );
    }
}

/// Uploads the tightly packed `bytes` of one mip `level`, `size` texels across and down.
fn write_level(
    context: &RenderContext,
    texture: &wgpu::Texture,
    level: u32,
    bytes: &[u8],
    (width, height): (u32, u32),
    texel: u32,
) {
    context.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture,
            mip_level: level,
            origin: wgpu::Origin3d::ZERO,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texel * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: data without a stride is the region's size in tight or padded rows,
    // and any other length is refused.
    #[test]
    fn default_stride_takes_tight_or_padded_rows() {
        let size = [32, 32, 1];
        let tight = 32 * 4;
        assert_eq!(
            default_stride(tight * 32, tight as u32, 32, size).ok(),
            Some(128)
        );
        assert_eq!(
            default_stride(256 * 32, tight as u32, 32, size).ok(),
            Some(256)
        );
        for len in [64 * 64 * 4, tight * 32 - 1, tight * 32 + 4] {
            assert!(
                matches!(
                    default_stride(len, tight as u32, 32, size),
                    Err(TextureError::InvalidInput(_))
                ),
                "{len} bytes should be refused"
            );
        }
        // Layers spaced 40 rows apart take the full spacing or the WebGPU minimum.
        let layered = [32, 32, 2];
        let least = (40 + 31) * tight + tight;
        assert_eq!(
            default_stride(least, tight as u32, 40, layered).ok(),
            Some(128)
        );
        assert_eq!(
            default_stride(tight * 80, tight as u32, 40, layered).ok(),
            Some(128)
        );
    }
}
