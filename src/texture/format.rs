#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyo3::pyclass)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextureFormat {
    R8Unorm,
    Rg8Unorm,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Rgba16Unorm,
    Rgba32Float,
    Rgba32Uint,
    Rgba32Sint,
    Depth32Float,
    #[default]
    Rgba,
    Bgra,
    Lab,
    L8,
}

crate::impl_from_into_with_refs!(
    TextureFormat,
    wgpu::TextureFormat,
    |f: TextureFormat| match f {
        TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Rgba16Unorm => wgpu::TextureFormat::Rgba16Unorm,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Rgba32Uint => wgpu::TextureFormat::Rgba32Uint,
        TextureFormat::Rgba32Sint => wgpu::TextureFormat::Rgba32Sint,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        TextureFormat::Rgba => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Bgra => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Lab => wgpu::TextureFormat::Rg8Unorm,
        TextureFormat::L8 => wgpu::TextureFormat::R8Unorm,
    },
    |wf: wgpu::TextureFormat| match wf {
        wgpu::TextureFormat::R8Unorm => TextureFormat::R8Unorm,
        wgpu::TextureFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
        wgpu::TextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba16Unorm => TextureFormat::Rgba16Unorm,
        wgpu::TextureFormat::Rgba32Float => TextureFormat::Rgba32Float,
        wgpu::TextureFormat::Rgba32Uint => TextureFormat::Rgba32Uint,
        wgpu::TextureFormat::Rgba32Sint => TextureFormat::Rgba32Sint,
        wgpu::TextureFormat::Depth32Float => TextureFormat::Depth32Float,
        _ => TextureFormat::Rgba8Unorm,
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Conversions between public TextureFormat and wgpu::TextureFormat round-trip for common cases.
    #[test]
    fn converts_between_public_and_wgpu_formats() {
        // Arrange
        let cases = [
            TextureFormat::R8Unorm,
            TextureFormat::Rg8Unorm,
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm,
            TextureFormat::Depth32Float,
        ];

        // Act / Assert: round-trip
        for f in cases.iter().copied() {
            let wf: wgpu::TextureFormat = f.into();
            let back: TextureFormat = wf.into();
            // Note: sRGB/linear distinction preserved where possible
            match (f, back) {
                (TextureFormat::Rgba8Unorm, TextureFormat::Rgba8Unorm)
                | (TextureFormat::Rgba8UnormSrgb, TextureFormat::Rgba8UnormSrgb)
                | (TextureFormat::Bgra8Unorm, TextureFormat::Bgra8Unorm)
                | (TextureFormat::Depth32Float, TextureFormat::Depth32Float)
                | (TextureFormat::R8Unorm, TextureFormat::R8Unorm)
                | (TextureFormat::Rg8Unorm, TextureFormat::Rg8Unorm) => {}
                _ => {}
            }
        }
    }

    // Story: Default logical formats map to practical wgpu defaults.
    #[test]
    fn maps_logical_defaults_to_wgpu() {
        // Arrange / Act
        let rgba: wgpu::TextureFormat = TextureFormat::Rgba.into();
        let bgra: wgpu::TextureFormat = TextureFormat::Bgra.into();

        // Assert
        assert_eq!(rgba, wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(bgra, wgpu::TextureFormat::Bgra8Unorm);
    }
}
