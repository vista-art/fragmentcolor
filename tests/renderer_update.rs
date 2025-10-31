use fragmentcolor::{Renderer, TextureFormat, TextureWriteOptions};

fn required_bytes(bpr: u32, height: u32, stride: u32, depth: u32, rows_per_image: u32) -> usize {
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
fn renderer_update_ok_and_unregister_not_found() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture([4u32, 4u32], TextureFormat::Rgba, None)
            .await
            .expect("create tex");
        let id = *tex.id();

        let w = 4u32;
        let h = 4u32;
        let stride = w * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let bpr = stride.div_ceil(align) * align;
        let req = required_bytes(bpr, h, stride, 1, h);
        let data = vec![0x7Fu8; req];

        // update_texture (defaults to whole)
        let opt = TextureWriteOptions::whole().with_bytes_per_row(bpr);
        r.update_texture_with(id, &data, opt).expect("update ok");

        // unregister and then update should fail
        r.unregister_texture(id).expect("unregister ok");
        let err = r
            .update_texture_with(
                id,
                &data,
                TextureWriteOptions::whole().with_bytes_per_row(bpr),
            )
            .expect_err("update after unregister must fail");
        let s = format!("{}", err);
        assert!(s.contains("not found") || s.to_lowercase().contains("not found"));
    });
}

#[test]
fn renderer_update_not_found() {
    let r = Renderer::new();
    let bogus = fragmentcolor::texture::TextureId(9_999_999);
    let err = r
        .update_texture(bogus, &[])
        .expect_err("missing id should err");
    let s = format!("{}", err);
    assert!(s.to_lowercase().contains("not found") || s.to_lowercase().contains("context"));
}
