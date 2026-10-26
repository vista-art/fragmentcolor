#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::renderer::error::RendererError;
use fragmentcolor::texture::TextureError;
use fragmentcolor::{Mipmap, Renderer, Texture, TextureData, TextureFormat};

// Wider than any device's 2D limit, in one row of RGBA pixels.
const WIDE: u32 = 131_072;

fn wide_pixels() -> Vec<u8> {
    vec![255; WIDE as usize * 4]
}

fn wide_png() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(WIDE, 1, image::Rgba([255, 255, 255, 255]));
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encode png");
    bytes
}

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
// area, are refused before they reach wgpu, though the bytes cover the size.
#[test]
fn sized_textures_check_their_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let pixels = wide_pixels();
        let result = renderer
            .create_texture((pixels.as_slice(), [WIDE, 1]))
            .await;
        assert!(
            refused(&result),
            "a raw size past the limit should be refused"
        );
        let result = renderer.create_texture((pixels.as_slice(), [0, 4])).await;
        assert!(
            refused(&result),
            "a raw size with no area should be refused"
        );
        renderer
            .create_texture((&pixels[..16], [2, 2]))
            .await
            .expect("a size that fits is made");
    });
}

// Story: a decoded image and a prepared mip chain wider than the device takes
// are refused before they reach wgpu, as an explicit size is.
#[test]
fn decoded_and_prepared_textures_check_their_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let result = renderer
            .create_texture(TextureData::Bytes(wide_png()))
            .await;
        assert!(
            refused(&result),
            "a decoded image past the limit should be refused"
        );

        let pixels = wide_pixels();
        let chain = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba8Unorm, [WIDE, 1]))
            .expect("Mipmap::build failed");
        let result = renderer.create_texture(chain).await;
        assert!(
            refused(&result),
            "a prepared chain past the limit should be refused"
        );
    });
}

// Story: the size in an image's header is checked before its pixels decode, so
// a file too wide for the device is refused as too wide even when its pixel data
// is cut short, where the decode ran first and failed on the missing data.
#[test]
fn decoded_images_check_the_header_first() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let mut png = wide_png();
        let data = png
            .windows(4)
            .position(|chunk| chunk == b"IDAT")
            .expect("an IDAT chunk");
        png.truncate(data + 6);
        let result = renderer.create_texture(TextureData::Bytes(png)).await;
        assert!(
            refused(&result),
            "a header past the limit should be refused, got {result:?}"
        );
    });
}

// Story: a storage texture with seed bytes too short for its size still
// reports the size, the error resize gives, when the size is past the limit.
#[test]
fn seeded_storage_textures_report_the_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let seed: &[u8] = &[0; 16];
        let result = renderer
            .create_storage_texture(([WIDE, 1], TextureFormat::Rgba, seed))
            .await;
        assert!(
            refused(&result),
            "the size, not the seed, should be refused"
        );
    });
}
