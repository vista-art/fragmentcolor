use fragmentcolor::{Renderer, TextureFormat};

#[test]
fn renderer_unregister_then_unregister_again_fails() {
    pollster::block_on(async move {
        let r = Renderer::new();
        let tex = r
            .create_storage_texture(([4u32, 4u32], TextureFormat::Rgba))
            .await
            .expect("create tex");
        let id = *tex.id();

        // First unregister succeeds.
        r.unregister_texture(id).expect("first unregister ok");

        // Second unregister of the same id must fail.
        let err = r
            .unregister_texture(id)
            .expect_err("second unregister of same id must fail");
        let s = format!("{}", err);
        assert!(s.to_lowercase().contains("not found") || s.to_lowercase().contains("context"));
    });
}

#[test]
fn renderer_unregister_unknown_id_fails() {
    let r = Renderer::new();
    let bogus = fragmentcolor::texture::TextureId { id: 9_999_999 };
    let err = r
        .unregister_texture(bogus)
        .expect_err("missing id should err");
    let s = format!("{}", err);
    assert!(s.to_lowercase().contains("not found") || s.to_lowercase().contains("context"));
}
