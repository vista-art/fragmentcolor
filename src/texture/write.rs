use crate::texture::{Texture, TextureError, TextureRegion};

pub(super) fn write(
    tex: &Texture,
    data: &[u8],
    mut region: TextureRegion,
) -> Result<(), TextureError> {
    // Validate usage supports COPY_DST
    if !tex.object.usage.contains(wgpu::TextureUsages::COPY_DST) {
        return Err(TextureError::Error(
            "Texture usage does not allow writes (missing COPY_DST)".into(),
        ));
    }

    let size = tex.object.size;
    let format = tex.object.format;
    let bpp = super::bytes_per_pixel(format);

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
    let row_stride = region.size[0].saturating_mul(bpp);
    let bytes_per_row = match region.bytes_per_row {
        Some(bpr) => {
            if bpr % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT != 0 {
                return Err(TextureError::Error(format!(
                    "bytes_per_row ({bpr}) must be a multiple of {}",
                    wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
                )));
            }
            if bpr < row_stride {
                return Err(TextureError::Error(
                    "bytes_per_row smaller than row stride".into(),
                ));
            }
            bpr
        }
        None => {
            // Align up to 256
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            row_stride.div_ceil(align) * align
        }
    };
    let rows_per_image = region.rows_per_image.unwrap_or(region.size[1]);
    if let Some(rpi) = region.rows_per_image
        && rpi < region.size[1]
    {
        return Err(TextureError::Error(
            "rows_per_image must be greater than or equal to region height".into(),
        ));
    }

    // Required minimum size per WebGPU: bpr*(h-1) + width_in_bytes + (depth-1)*rows_per_image*bpr
    let width_in_bytes = (region.size[0] as u64).saturating_mul(bpp as u64);
    let required = (bytes_per_row as u64)
        .saturating_mul(region.size[1].saturating_sub(1) as u64)
        .saturating_add(width_in_bytes)
        .saturating_add(
            (region.size[2].saturating_sub(1) as u64)
                .saturating_mul(rows_per_image as u64)
                .saturating_mul(bytes_per_row as u64),
        );

    if (data.len() as u64) < required {
        return Err(TextureError::Error(
            "Input data smaller than required for layout".into(),
        ));
    }

    // Write
    tex.context.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &tex.object.inner,
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
