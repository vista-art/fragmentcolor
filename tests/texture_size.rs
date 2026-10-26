#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::renderer::error::RendererError;
use fragmentcolor::texture::TextureError;
use fragmentcolor::{Renderer, Texture, TextureFormat};

fn refused(result: &Result<Texture, RendererError>) -> bool {
    matches!(
        result,
        Err(RendererError::TextureError(TextureError::InvalidInput(_)))
    )
}

// Story: a storage texture past the device limit, with no area or with no
// layers is refused before it reaches wgpu, and one with layers within the
// limit is made.
#[test]
fn storage_textures_check_their_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        for size in [[u32::MAX, 8, 1], [0, 8, 1], [8, 8, 0], [8, 8, u32::MAX]] {
            let result = renderer
                .create_storage_texture((size, TextureFormat::Rgba))
                .await;
            assert!(refused(&result), "{size:?} should be refused");
        }

        let layered = renderer
            .create_storage_texture(([8, 8, 4], TextureFormat::Rgba))
            .await
            .expect("layers within the limit are made");
        let size = layered.size();
        assert_eq!(
            [size.width, size.height, size.depth.unwrap_or(1)],
            [8, 8, 4]
        );
    });
}

// Story: raw pixels with an explicit size past the device limit, or with no
// area, are refused before they reach wgpu.
#[test]
fn sized_textures_check_their_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let pixels: &[u8] = &[255; 16];
        for size in [[u32::MAX, 1], [0, 4]] {
            let result = renderer.create_texture((pixels, size)).await;
            assert!(refused(&result), "{size:?} should be refused");
        }
        renderer
            .create_texture((pixels, [2, 2]))
            .await
            .expect("a size that fits is made");
    });
}
