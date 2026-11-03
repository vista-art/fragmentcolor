#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::TextureError;
use fragmentcolor::{InitializationError, Renderer, Target};

// Story: Creating a target from a headless window should fall back to a texture-backed target
// and allow image readback for validation in CI.
#[test]
fn creates_texture_backed_target_from_headless_window() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let window = fragmentcolor::HeadlessWindow::new([32, 24]);

        // Act
        let target = renderer
            .create_target(window)
            .await
            .expect("create_target(headless) should succeed");

        // Assert
        let s = target.size();
        assert_eq!([s.width, s.height], [32, 24]);

        // Should provide a readable image buffer (texture-backed path)
        let img = target.get_image().await.expect("read back");
        assert!(
            !img.is_empty(),
            "headless window target should be texture-backed and readable"
        );
    });
}

// Story: a window past the device limit is refused with the size error, on the
// texture fallback a headless run takes too, and a window with no area yet still
// gets a target of one pixel.
#[test]
fn create_target_checks_the_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let refused = renderer
            .create_target(fragmentcolor::HeadlessWindow::new([u32::MAX, 8]))
            .await;
        assert!(matches!(
            refused,
            Err(InitializationError::TextureError(
                TextureError::InvalidInput(_)
            ))
        ));

        let target = renderer
            .create_target(fragmentcolor::HeadlessWindow::new([0, 8]))
            .await
            .expect("a window with no area still gets a target");
        let size = target.size();
        assert_eq!([size.width, size.height], [1, 8]);
    });
}
