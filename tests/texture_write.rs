use fragmentcolor::{Renderer, TextureFormat, TextureRegion};

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
            .create_storage_texture((size, TextureFormat::Rgba))
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

        let region = TextureRegion::from([w, h]).with_stride(bpr);
        tex.write_region(&data, region).expect("write full ok");
    });
}

#[test]
fn texture_write_explicit_tight_stride_ok() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([4u32, 2u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        // A stride of 16 bytes is not a multiple of 256, which write_texture allows
        let stride = 4u32 * 4;
        let data = vec![0u8; (stride * 2) as usize];
        let region = TextureRegion::from([4, 2]).with_stride(stride);
        tex.write_region(&data, region)
            .expect("tight explicit stride must succeed");
    });
}

#[test]
fn texture_write_tightly_packed_round_trip() {
    pollster::block_on(async move {
        let r = Renderer::new();
        // 3 * 4 = 12 bytes per row, far from a multiple of 256
        let tex = r
            .create_storage_texture(([3u32, 2u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let data: Vec<u8> = (0..3u8 * 2 * 4).collect();
        tex.write(&data).expect("tightly packed write");

        let image = tex.get_image().await.expect("read back");
        assert_eq!(image, data);
    });
}

#[test]
fn texture_write_padded_rows_without_stride() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([3u32, 2u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        // Each 12-byte row padded to 256, with exactly height * 256 bytes
        let tight: Vec<u8> = (0..3u8 * 2 * 4).collect();
        let mut padded = vec![0xEEu8; 256 * 2];
        padded[..12].copy_from_slice(&tight[..12]);
        padded[256..256 + 12].copy_from_slice(&tight[12..]);
        tex.write(&padded).expect("padded write");

        let image = tex.get_image().await.expect("read back");
        assert_eq!(image, tight);
    });
}

#[test]
fn texture_write_region_padded_rows_without_stride() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([4u32, 3u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let clear = vec![0u8; 4 * 3 * 4];
        tex.write(&clear).expect("clear");

        // A 2x2 rectangle at (1, 1), each 8-byte row padded to 256
        let mut padded = vec![0xEEu8; 256 * 2];
        padded[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        padded[256..256 + 8].copy_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        tex.write_region(&padded, [1u32, 1, 2, 2])
            .expect("padded region write");

        let image = tex.get_image().await.expect("read back");
        let row = |y: usize| &image[y * 16..(y + 1) * 16];
        assert_eq!(row(0), &[0u8; 16]);
        assert_eq!(row(1), &[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0]);
        assert_eq!(
            row(2),
            &[0, 0, 0, 0, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0]
        );
    });
}

#[test]
fn texture_write_tightly_packed_too_small() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([3u32, 2u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let data = vec![0u8; 3 * 2 * 4 - 1];
        let err = tex.write(&data).expect_err("one byte short must err");
        assert!(
            matches!(err, fragmentcolor::texture::TextureError::InvalidInput(_)),
            "a short buffer is invalid input, got {err}"
        );
    });
}

#[test]
fn texture_write_region_tightly_packed_round_trip() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let (w, h) = (5usize, 3usize);
        let tex = r
            .create_storage_texture(([w as u32, h as u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let clear = vec![0u8; w * h * 4];
        tex.write(&clear).expect("clear");

        // A 3x2 rectangle at (1, 1), packed at 12 bytes per row
        let (x0, y0, rw, rh) = (1usize, 1usize, 3usize, 2usize);
        let patch: Vec<u8> = (1..=(rw * rh * 4) as u8).collect();
        tex.write_region(&patch, [x0 as u32, y0 as u32, rw as u32, rh as u32])
            .expect("tightly packed region write");

        let image = tex.get_image().await.expect("read back");
        for y in 0..h {
            for x in 0..w {
                let got = &image[(y * w + x) * 4..][..4];
                let inside = (x0..x0 + rw).contains(&x) && (y0..y0 + rh).contains(&y);
                if inside {
                    let i = ((y - y0) * rw + (x - x0)) * 4;
                    assert_eq!(got, &patch[i..i + 4], "pixel ({x}, {y})");
                } else {
                    assert_eq!(got, &[0u8; 4], "pixel ({x}, {y}) outside the region");
                }
            }
        }
    });
}

#[test]
fn texture_write_bpr_smaller_than_stride() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([128u32, 1u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let width = 128u32;
        let stride = width * 4;
        // Force bpr smaller than stride while aligned: stride=512, pick bpr=256
        let bpr = 256u32; // aligned and < stride(512)
        let data = vec![0xCDu8; stride as usize];
        let region = TextureRegion::from([width, 1]).with_stride(bpr);
        let err = tex
            .write_region(&data, region)
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
            .create_storage_texture(([8u32, 2u32], TextureFormat::Rgba))
            .await
            .expect("create storage tex");
        let width = 8u32;
        let pixel = 4u32;
        let stride = width * pixel; // 32
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; // 256
        let bpr = stride.div_ceil(align) * align; // 256
        // Provide only one row of data; required would be 256*(h-1)+stride = 256+32=288
        let too_small = vec![0u8; stride as usize];
        let region = TextureRegion::from([width, 2]).with_stride(bpr);
        let err = tex
            .write_region(&too_small, region)
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
            .write_region(&data, TextureRegion::from([w, h]).with_stride(bpr))
            .expect_err("no COPY_DST should fail");
        let s = format!("{}", err);
        assert!(s.contains("does not allow writes") || s.contains("COPY_DST"));
    });
}
