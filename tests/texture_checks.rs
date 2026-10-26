#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::renderer::error::RendererError;
use fragmentcolor::texture::TextureError;
use fragmentcolor::{
    Renderer, Texture, TextureFormat, TextureInput, TextureOptions, TextureUsage,
};

fn refused(result: &Result<Texture, RendererError>) -> bool {
    matches!(
        result,
        Err(RendererError::TextureError(TextureError::InvalidInput(_)))
    )
}

/// Raw RGBA pixels for a 4x4 texture, with the usage bits `usage`.
fn pixels_with_usage(usage: u32) -> TextureInput {
    TextureInput {
        data: vec![255; 4 * 4 * 4].into(),
        options: TextureOptions {
            size: Some([4, 4].into()),
            usage: Some(usage),
            ..Default::default()
        },
    }
}

/// An empty 4x4 storage texture of `format`, with the usage bits `usage`.
fn storage_with_usage(format: TextureFormat, usage: Option<u32>) -> TextureInput {
    TextureInput {
        data: Default::default(),
        options: TextureOptions {
            size: Some([4, 4].into()),
            format,
            usage,
            ..Default::default()
        },
    }
}

// Story: a usage mask takes only the five TextureUsage flags, so wgpu-only bits
// such as TRANSIENT_ATTACHMENT and STORAGE_ATOMIC, and unknown bits, are refused
// on both entry points instead of reaching wgpu or being dropped.
#[test]
fn usage_takes_only_the_named_flags() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        for bits in [32, 64, 65536] {
            let result = renderer.create_texture(pixels_with_usage(bits)).await;
            assert!(refused(&result), "create_texture usage {bits}");
            let input = storage_with_usage(TextureFormat::Rgba, Some(bits));
            let result = renderer.create_storage_texture(input).await;
            assert!(refused(&result), "create_storage_texture usage {bits}");
        }

        // create_texture always adds TEXTURE_BINDING | COPY_DST, so 0 is fine there.
        renderer
            .create_texture(pixels_with_usage(0))
            .await
            .expect("create_texture takes an empty extra usage");
        let input = storage_with_usage(TextureFormat::Rgba, Some(0));
        let result = renderer.create_storage_texture(input).await;
        assert!(refused(&result), "create_storage_texture with no usage");
    });
}

// Story: a storage texture in a format that takes no storage, and a depth texture
// with layers, are refused before they reach wgpu, which only takes 2D depth textures.
#[test]
fn storage_textures_check_their_format() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        for format in [TextureFormat::Depth32Float, TextureFormat::Rgba8UnormSrgb] {
            let result = renderer.create_storage_texture(([4, 4], format)).await;
            assert!(refused(&result), "{format:?} takes no storage");
        }

        let sampled = TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_SRC;
        let layered = TextureInput {
            data: Default::default(),
            options: TextureOptions {
                size: Some([4, 4, 2].into()),
                format: TextureFormat::Depth32Float,
                ..Default::default()
            }
            .with_usage(sampled),
        };
        let result = renderer.create_storage_texture(layered).await;
        assert!(refused(&result), "a depth texture has one layer");

        let input = storage_with_usage(TextureFormat::Depth32Float, Some(sampled.bits));
        renderer
            .create_storage_texture(input)
            .await
            .expect("a flat depth texture without storage is made");
    });
}
