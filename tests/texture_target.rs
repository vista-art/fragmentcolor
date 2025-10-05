#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Shader, Target};

// Story: Rendering into an offscreen texture should succeed and produce readable pixels.
#[test]
fn renders_to_texture_and_reads_back() {
    // Arrange
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target([16, 16]))
        .expect("create_texture_target failed");
    let shader = Shader::default();

    // Act
    renderer.render(&shader, &target).expect("render failed");

    // Assert
    let img = target.get_image();
    assert!(!img.is_empty(), "image readback should not be empty");
}
