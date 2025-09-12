#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Target};

#[test]
fn create_target_from_headless_window_falls_back_to_texture() {
    let renderer = Renderer::new();
    let window = pollster::block_on(fragmentcolor::headless_window([32, 24]));
    let target = pollster::block_on(renderer.create_target(window))
        .expect("create_target(headless) should succeed");

    let s = target.size();
    assert_eq!([s.width, s.height], [32, 24]);

    // Should provide a readable image buffer (texture-backed path)
    let img = target.get_image();
    assert!(
        !img.is_empty(),
        "headless window target should be texture-backed and readable"
    );
}
