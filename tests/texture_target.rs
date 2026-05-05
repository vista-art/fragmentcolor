#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Shader, Target};

// Story: Rendering into an offscreen texture should succeed and produce readable pixels.
#[test]
fn renders_to_texture_and_reads_back() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([16, 16])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::default();

        // Act
        renderer.render(&shader, &target).expect("render failed");

        // Assert
        let img = target.get_image().await;
        assert!(!img.is_empty(), "image readback should not be empty");
    });
}
