use crate::texture::{Texture, TextureError, TextureWriteOptions};

pub(super) fn write(
    tex: &Texture,
    data: &[u8],
    mut opt: TextureWriteOptions,
) -> Result<(), TextureError> {
    // Validate usage supports COPY_DST
    if !tex.object.usage.contains(wgpu::TextureUsages::COPY_DST) {
        return Err(TextureError::CreateTextureError(
            "Texture usage does not allow writes (missing COPY_DST)".into(),
        ));
    }

    let size = tex.object.size;
    let format = tex.object.format;
    let bpp = super::bytes_per_pixel(format);

    // Infer full-region if caller used whole() defaults
    if opt.size_width == 0 || opt.size_height == 0 || opt.size_depth == 0 {
        opt.size_width = size.width;
        opt.size_height = size.height;
        opt.size_depth = size.depth_or_array_layers.max(1);
    }

    // Compute or validate layout
    let row_stride = opt.size_width.saturating_mul(bpp);
    let bytes_per_row = match opt.bytes_per_row {
        Some(bpr) => {
            if bpr % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT != 0 {
                return Err(TextureError::CreateTextureError(format!(
                    "bytes_per_row ({bpr}) must be a multiple of {}",
                    wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
                )));
            }
            if bpr < row_stride {
                return Err(TextureError::CreateTextureError(
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
        return Err(TextureError::CreateTextureError(
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
