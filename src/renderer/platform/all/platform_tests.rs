// Separate file so items-after-test-module lint is not triggered
use super::{choose_surface_format, choose_surface_format_from};
use crate::renderer::error::InitializationError;

#[test]
fn chooses_surface_format_with_linear_preference() {
    // Arrange: linear exists alongside sRGB
    let formats = vec![
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    // Act / Assert: prefer linear BGRA over sRGB RGBA
    let f = choose_surface_format_from(&formats);
    assert_eq!(f, Some(wgpu::TextureFormat::Bgra8Unorm));

    // Arrange: no linear, only sRGB
    let formats2 = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
    let f2 = choose_surface_format_from(&formats2);
    assert_eq!(f2, Some(wgpu::TextureFormat::Rgba8UnormSrgb));

    // Arrange: an empty list has no format to choose
    let empty: Vec<wgpu::TextureFormat> = vec![];
    let f3 = choose_surface_format_from(&empty);
    assert_eq!(f3, None);
}

// Story: wgpu reports default capabilities for a surface the adapter cannot present to,
// such as a second WebGL2 canvas. Configuring it anyway only logs UnsupportedQueueFamily.
#[test]
fn rejects_a_surface_without_formats() {
    // Arrange
    let capabilities = wgpu::SurfaceCapabilities::default();

    // Act
    let result = choose_surface_format(&capabilities);

    // Assert
    assert!(
        matches!(&result, Err(InitializationError::Error(message)) if message.contains("lists no formats")),
        "got {result:?}"
    );
}
