// Separate file so items-after-test-module lint is not triggered
use super::choose_surface_format_from;

#[test]
fn chooses_surface_format_with_linear_preference() {
    // Arrange: linear exists alongside sRGB
    let formats = vec![
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    // Act / Assert: prefer linear BGRA over sRGB RGBA
    let f = choose_surface_format_from(&formats);
    assert_eq!(f, wgpu::TextureFormat::Bgra8Unorm);

    // Arrange: no linear, only sRGB
    let formats2 = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
    let f2 = choose_surface_format_from(&formats2);
    assert_eq!(f2, wgpu::TextureFormat::Rgba8UnormSrgb);

    // Arrange: empty list falls back to Rgba8Unorm
    let empty: Vec<wgpu::TextureFormat> = vec![];
    let f3 = choose_surface_format_from(&empty);
    assert_eq!(f3, wgpu::TextureFormat::Rgba8Unorm);
}
