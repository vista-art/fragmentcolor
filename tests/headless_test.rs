#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Shader, Target};

#[test]
fn render_to_texture_target_and_readback() {
    let renderer = Renderer::new();
    let target = pollster::block_on(renderer.create_texture_target([16, 16]))
        .expect("create_texture_target failed");

    let shader = Shader::default();
    renderer.render(&shader, &target).expect("render failed");

    // Should be a TextureTarget under the hood, providing readback
    let img = target.get_image();
    assert!(!img.is_empty(), "image readback should not be empty");
}
