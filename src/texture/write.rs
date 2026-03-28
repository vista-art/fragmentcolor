use crate::texture::{Texture, TextureError, TextureWriteOptions};

pub(super) fn write(
    tex: &Texture,
    data: &[u8],
    mut opt: TextureWriteOptions,
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

    // Infer per-dimension if caller used whole() defaults; respect origin
    if opt.size_width == 0 {
        opt.size_width = size.width.saturating_sub(opt.origin_x);
    }
    if opt.size_height == 0 {
        opt.size_height = size.height.saturating_sub(opt.origin_y);
    }
    if opt.size_depth == 0 {
        let d = size.depth_or_array_layers.max(1);
        // For 2D textures depth is 1; clamp to at least 1 after subtracting origin
        opt.size_depth = d.saturating_sub(opt.origin_z).max(1);
    }

    // Validate non-zero sizes
    if opt.size_width == 0 || opt.size_height == 0 || opt.size_depth == 0 {
        return Err(TextureError::Error(
            "Write size must be non-zero in all dimensions".into(),
        ));
    }

    // Bounds check: origin and region must lie within texture extent
    if opt.origin_x >= size.width
        || opt.origin_y >= size.height
        || opt.origin_z >= size.depth_or_array_layers
    {
        return Err(TextureError::Error("Origin out of texture bounds".into()));
    }
    let end_x = opt.origin_x.saturating_add(opt.size_width);
    let end_y = opt.origin_y.saturating_add(opt.size_height);
    let end_z = opt.origin_z.saturating_add(opt.size_depth);
    if end_x > size.width || end_y > size.height || end_z > size.depth_or_array_layers {
        return Err(TextureError::Error(
            "Write region exceeds texture bounds".into(),
        ));
    }

    // Compute or validate layout
    let row_stride = opt.size_width.saturating_mul(bpp);
    let bytes_per_row = match opt.bytes_per_row {
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
    let rows_per_image = opt.rows_per_image.unwrap_or(opt.size_height);
    if let Some(rpi) = opt.rows_per_image {
        if rpi < opt.size_height {
            return Err(TextureError::Error(
                "rows_per_image must be greater than or equal to size_height".into(),
            ));
        }
    }

    // Required minimum size per WebGPU: bpr*(h-1) + width_in_bytes + (depth-1)*rows_per_image*bpr
    let width_in_bytes = (opt.size_width as u64).saturating_mul(bpp as u64);
    let required = (bytes_per_row as u64)
        .saturating_mul(opt.size_height.saturating_sub(1) as u64)
        .saturating_add(width_in_bytes)
        .saturating_add(
            (opt.size_depth.saturating_sub(1) as u64)
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
                x: opt.origin_x,
                y: opt.origin_y,
                z: opt.origin_z,
            },
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(rows_per_image),
        },
        wgpu::Extent3d {
            width: opt.size_width,
            height: opt.size_height,
            depth_or_array_layers: opt.size_depth,
        },
    );

    Ok(())
}
