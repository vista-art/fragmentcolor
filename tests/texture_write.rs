use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};

fn required_bytes(bpr: u32, height: u32, stride: u32, depth: u32, rows_per_image: u32) -> usize {
    // bpr*(h-1) + stride + (depth-1)*rows_per_image*bpr
    (bpr as u64)
        .saturating_mul(height.saturating_sub(1) as u64)
        .saturating_add(stride as u64)
        .saturating_add(
            (depth.saturating_sub(1) as u64)
                .saturating_mul(rows_per_image as u64)
                .saturating_mul(bpr as u64),
        ) as usize
}

#[test]
fn texture_write_full_ok() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let size = [2u32, 2u32];
        let tex = r
            .create_storage_texture(size, TextureFormat::Rgba, None)
            .await
            .expect("create storage tex");

        let w = 2u32;
        let h = 2u32;
        let pixel = 4u32;
        let stride = w * pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let bpr = stride.div_ceil(align) * align;
        let req = required_bytes(bpr, h, stride, 1, h);
        let data = vec![0xABu8; req];

        let opt = TextureWriteOptions::whole().with_bytes_per_row(bpr);
        tex.write_with(&data, opt).expect("write full ok");
    });
}

#[test]
fn texture_write_invalid_bpr_alignment() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture([4u32, 1u32], TextureFormat::Rgba, None)
            .await
            .expect("create storage tex");
        let width = 4u32;
        let stride = width * 4;
        // Intentionally use stride (not 256-aligned) for bpr
        let data = vec![0u8; stride as usize];
        let opt = TextureWriteOptions::whole().with_bytes_per_row(stride);
        let err = tex.write_with(&data, opt).expect_err("bad bpr must error");
        let s = format!("{}", err);
        assert!(s.contains("bytes_per_row") || s.contains("multiple"));
    });
}

#[test]
fn texture_write_bpr_smaller_than_stride() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture([128u32, 1u32], TextureFormat::Rgba, None)
            .await
            .expect("create storage tex");
        let width = 128u32;
        let stride = width * 4;
        // Force bpr smaller than stride while aligned: stride=512, pick bpr=256
        let bpr = 256u32; // aligned and < stride(512)
        let data = vec![0xCDu8; stride as usize];
        let opt = TextureWriteOptions::whole().with_bytes_per_row(bpr);
        let err = tex
            .write_with(&data, opt)
            .expect_err("bpr < stride must err");
        let s = format!("{}", err);
        assert!(s.contains("bytes_per_row smaller"));
    });
}

#[test]
fn texture_write_data_too_small() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture([8u32, 2u32], TextureFormat::Rgba, None)
            .await
            .expect("create storage tex");
        let width = 8u32;
        let pixel = 4u32;
        let stride = width * pixel; // 32
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; // 256
        let bpr = stride.div_ceil(align) * align; // 256
        // Provide only one row of data; required would be 256*(h-1)+stride = 256+32=288
        let too_small = vec![0u8; stride as usize];
        let opt = TextureWriteOptions::whole().with_bytes_per_row(bpr);
        let err = tex
            .write_with(&too_small, opt)
            .expect_err("too small must err");
        let s = format!("{}", err);
        assert!(s.contains("smaller") || s.contains("required"));
    });
}

#[test]
fn texture_write_no_copy_dst_usage_errors() {
    pollster::block_on(async move {
        let r = Renderer::new();
        // Create a texture target; its underlying texture lacks COPY_DST
        let tgt = r
            .create_texture_target([2u32, 2u32])
            .await
            .expect("create texture target");
        let tex = tgt.texture();
        let w = 2u32;
        let h = 2u32;
        let stride = w * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let bpr = stride.div_ceil(align) * align;
        let req = required_bytes(bpr, h, stride, 1, h);
        let data = vec![0u8; req];
        let err = tex
            .write_with(&data, TextureWriteOptions::whole().with_bytes_per_row(bpr))
            .expect_err("no COPY_DST should fail");
        let s = format!("{}", err);
        assert!(s.contains("does not allow writes") || s.contains("COPY_DST"));
    });
}
