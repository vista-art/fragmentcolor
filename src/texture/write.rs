use crate::texture::{Texture, TextureError, TextureRegion};

pub(super) fn write(
    tex: &Texture,
    data: &[u8],
    mut region: TextureRegion,
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
    let bpp = super::texel_size(format)?;

    // Infer per-dimension size when caller used the "whole texture" defaults
    // (TextureRegion::default() has size [0, 0, 0]); always respect origin.
    if region.size[0] == 0 {
        region.size[0] = size.width.saturating_sub(region.origin[0]);
    }
    if region.size[1] == 0 {
        region.size[1] = size.height.saturating_sub(region.origin[1]);
    }
    if region.size[2] == 0 {
        let d = size.depth_or_array_layers.max(1);
        // For 2D textures depth is 1; clamp to at least 1 after subtracting origin.
        region.size[2] = d.saturating_sub(region.origin[2]).max(1);
    }

    // Validate non-zero sizes
    if region.size[0] == 0 || region.size[1] == 0 || region.size[2] == 0 {
        return Err(TextureError::Error(
            "Write size must be non-zero in all dimensions".into(),
        ));
    }

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

    let required = least_bytes(bytes_per_row, row_stride, rows_per_image, region.size);
    if (data.len() as u64) < required {
        return Err(TextureError::Error(
            "Input data smaller than required for layout".into(),
        ));
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

/// The fewest bytes WebGPU reads for a region of `size` at `bytes_per_row`:
/// every full row but the last, then the last row's `row_stride`.
fn least_bytes(bytes_per_row: u32, row_stride: u32, rows_per_image: u32, size: [u32; 3]) -> u64 {
    let rows = (size[2].saturating_sub(1) as u64)
        .saturating_mul(rows_per_image as u64)
        .saturating_add(size[1].saturating_sub(1) as u64);
    (bytes_per_row as u64)
        .saturating_mul(rows)
        .saturating_add(row_stride as u64)
}

/// Row stride for data of `len` bytes passed without `bytes_per_row`.
/// Rows are tightly packed, since `Queue::write_texture` takes any stride,
/// and data sized exactly for rows padded to 256 bytes reads at the padded stride.
/// Any other length is an error, so a buffer for a different region never uploads cropped.
fn default_stride(
    len: usize,
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
    let len = len as u64;
    if len == full(row_stride) || len == least_bytes(row_stride, row_stride, rows_per_image, size)
    {
        return Ok(row_stride);
    }
    if len == full(padded) {
        return Ok(padded);
    }
    let [width, height, depth] = size;
    Err(TextureError::InvalidInput(format!(
        "{len} bytes don't fit a {width}x{height}x{depth} region, which takes {} in tight rows or {} in rows padded to {align} bytes; give the region a bytes per row for any other layout",
        full(row_stride),
        full(padded)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: data without a stride is the region's size in tight or padded rows,
    // and any other length is refused, where it cropped or read past the region.
    #[test]
    fn default_stride_takes_tight_or_padded_rows() {
        let size = [32, 32, 1];
        let tight = 32 * 4;
        assert_eq!(default_stride(tight * 32, tight as u32, 32, size).ok(), Some(128));
        assert_eq!(default_stride(256 * 32, tight as u32, 32, size).ok(), Some(256));
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
        assert_eq!(default_stride(least, tight as u32, 40, layered).ok(), Some(128));
        assert_eq!(default_stride(tight * 80, tight as u32, 40, layered).ok(), Some(128));
    }
}
