#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Target};

// Story: Creating a target from a headless window should fall back to a texture-backed target
// and allow image readback for validation in CI.
#[test]
fn creates_texture_backed_target_from_headless_window() {
    // Arrange
    let renderer = Renderer::new();
    let window = fragmentcolor::headless_window([32, 24]);

    // Act
    let target = pollster::block_on(renderer.create_target(window))
        .expect("create_target(headless) should succeed");

    // Assert
    let s = target.size();
    assert_eq!([s.width, s.height], [32, 24]);

    // Should provide a readable image buffer (texture-backed path)
    let img = target.get_image();
    assert!(
        !img.is_empty(),
        "headless window target should be texture-backed and readable"
    );
}
