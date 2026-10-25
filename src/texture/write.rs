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

    let size = object.size;
    let format = object.format;
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
        None => default_stride(data.len(), row_stride, rows_per_image, region.size[2]),
    };

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

/// Row stride for data passed without `bytes_per_row`.
/// Rows are tightly packed, since `Queue::write_texture` takes any stride.
/// Data sized exactly for rows padded to 256 bytes reads at the padded stride.
fn default_stride(len: usize, row_stride: u32, rows_per_image: u32, depth: u32) -> u32 {
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = row_stride.div_ceil(align).saturating_mul(align);
    let padded_len = (padded as u64)
        .saturating_mul(rows_per_image as u64)
        .saturating_mul(depth as u64);
    if padded != row_stride && len as u64 == padded_len {
        padded
    } else {
        row_stride
    }
}
