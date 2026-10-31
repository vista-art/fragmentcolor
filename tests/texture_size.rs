#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::RendererError;
use fragmentcolor::TextureError;
use fragmentcolor::{Mipmap, Renderer, Size, Target, Texture, TextureData, TextureFormat};

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

fn small_png() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encode png");
    bytes
}

// Story: every path that makes a plain 2D texture reports its size with no
// depth, so two sizes of one shape compare equal in every binding, where some
// paths reported a depth of 1 and others none. A 3D texture keeps its depth.
#[test]
fn every_2d_path_reports_one_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let flat = Size::new(2, 2, None);
        let pixels = vec![255u8; 2 * 2 * 4];
        let chain =
            Mipmap::build((pixels.as_slice(), TextureFormat::Rgba, [2, 2])).expect("Mipmap::build");
        let created = [
            (
                "raw pixels and [w, h]",
                renderer.create_texture((pixels.as_slice(), [2, 2])).await,
            ),
            (
                "raw pixels and [w, h, 1]",
                renderer
                    .create_texture((pixels.as_slice(), [2, 2, 1]))
                    .await,
            ),
            (
                "an encoded image",
                renderer.create_texture(small_png()).await,
            ),
            ("a Mipmap", renderer.create_texture(chain).await),
            (
                "a storage texture",
                renderer
                    .create_storage_texture(([2, 2], TextureFormat::Rgba))
                    .await,
            ),
            (
                "a storage texture of depth 1",
                renderer
                    .create_storage_texture(([2, 2, 1], TextureFormat::Rgba))
                    .await,
            ),
        ];
        for (label, texture) in created {
            let texture = texture.unwrap_or_else(|error| panic!("{label}: {error}"));
            assert_eq!(texture.size(), flat, "{label}");
        }
        let depth = renderer
            .create_depth_texture([2, 2])
            .await
            .expect("create_depth_texture");
        assert_eq!(depth.size(), flat, "a depth texture");
        let target = renderer
            .create_texture_target([2, 2])
            .await
            .expect("create_texture_target");
        assert_eq!(target.size(), flat, "a texture target");
        assert_eq!(target.texture().size(), flat, "a texture target's texture");

        depth.resize([2, 2, 1]).expect("resize");
        assert_eq!(depth.size(), flat, "a resized texture");

        let deep = renderer
            .create_storage_texture(([2, 2, 3], TextureFormat::Rgba))
            .await
            .expect("a 3D storage texture");
        assert_eq!(deep.size(), Size::new(2, 2, Some(3)), "a 3D texture");
    });
}
