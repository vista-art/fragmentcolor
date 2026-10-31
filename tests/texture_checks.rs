#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::RendererError;
use fragmentcolor::TextureError;
use fragmentcolor::{
    Mipmap, Renderer, Size, Texture, TextureFormat, TextureInput, TextureOptions, TextureUsage,
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

/// Encodes `image` as a file in `format` would hold it.
fn encode(image: image::DynamicImage, format: image::ImageFormat) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut bytes), format)
        .expect("encode image");
    bytes
}

/// Options that read the texture back, in `format`.
fn readable(format: TextureFormat) -> TextureOptions {
    TextureOptions {
        format,
        mipmaps: false,
        ..Default::default()
    }
    .with_usage(TextureUsage::COPY_SRC)
}

fn halves(bytes: &[u8]) -> Vec<f32> {
    let half = |word: &[u8; 2]| half::f16::from_le_bytes(*word).to_f32();
    bytes.as_chunks::<2>().0.iter().map(half).collect()
}

fn floats(bytes: &[u8]) -> Vec<f32> {
    let word = |word: &[u8; 4]| f32::from_le_bytes(*word);
    bytes.as_chunks::<4>().0.iter().map(word).collect()
}

fn close(got: &[f32], want: &[f32]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| (a - b).abs() < 0.02)
}

// Story: an HDR file decodes into Rgba16Float with its bright values intact, a
// PNG fills an explicit float format, and an integer format refuses a decoded
// image, where each uploaded RGBA8 bytes into the wrong layout.
#[test]
fn decoded_images_fill_float_formats() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let sky = image::Rgb32FImage::from_raw(2, 1, vec![2.0, 0.5, 0.25, 1.0, 1.0, 1.0])
            .expect("sky pixels");
        let hdr = encode(sky.into(), image::ImageFormat::Hdr);
        let texture = renderer
            .create_texture((hdr, readable(TextureFormat::Rgba)))
            .await
            .expect("an HDR file decodes");
        let image = texture.get_image().await.expect("read back");
        let want = [2.0, 0.5, 0.25, 1.0, 1.0, 1.0, 1.0, 1.0];
        assert!(
            close(&halves(&image), &want),
            "HDR read back {:?}",
            halves(&image)
        );

        let pixel = image::RgbaImage::from_raw(1, 1, vec![255, 0, 51, 255]).expect("pixel");
        let png = encode(pixel.into(), image::ImageFormat::Png);
        let want = [1.0, 0.0, 0.2, 1.0];
        let texture = renderer
            .create_texture((png.clone(), readable(TextureFormat::Rgba32Float)))
            .await
            .expect("a PNG fills Rgba32Float");
        let image = texture.get_image().await.expect("read back");
        assert!(
            close(&floats(&image), &want),
            "Rgba32Float {:?}",
            floats(&image)
        );
        let texture = renderer
            .create_texture((png.clone(), readable(TextureFormat::Rgba16Float)))
            .await
            .expect("a PNG fills Rgba16Float");
        let image = texture.get_image().await.expect("read back");
        assert!(
            close(&halves(&image), &want),
            "Rgba16Float {:?}",
            halves(&image)
        );

        for format in [TextureFormat::Rgba32Uint, TextureFormat::Rgba32Sint] {
            let result = renderer.create_texture((png.clone(), format)).await;
            assert!(refused(&result), "a decoded image into {format:?}");
        }
    });
}

// Story: an HDR value past the half float range saturates at 65504 in the
// default Rgba16Float, where it became infinity, and an explicit Rgba32Float
// keeps it whole.
#[test]
fn hdr_values_saturate_in_half_floats() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let sun = image::Rgb32FImage::from_raw(1, 1, vec![131072.0, 1.0, 0.5]).expect("sun");
        let hdr = encode(sun.into(), image::ImageFormat::Hdr);
        let texture = renderer
            .create_texture((hdr.clone(), readable(TextureFormat::Rgba)))
            .await
            .expect("an HDR file decodes");
        let image = halves(&texture.get_image().await.expect("read back"));
        assert_eq!(image[0], 65504.0, "the sun saturates, got {image:?}");

        let texture = renderer
            .create_texture((hdr, readable(TextureFormat::Rgba32Float)))
            .await
            .expect("an HDR file fills Rgba32Float");
        let image = floats(&texture.get_image().await.expect("read back"));
        assert_eq!(image[0], 131072.0, "Rgba32Float keeps the full value");
    });
}

// Story: a decoded image in a Bgra8 format stores blue first, so its colors
// sample right, while raw bytes, already in the format's order, upload as given.
#[test]
fn decoded_images_fill_bgra_in_order() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let red = image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).expect("pixel");
        let png = encode(red.into(), image::ImageFormat::Png);
        let blue_first = [0, 0, 255, 255];

        let texture = renderer
            .create_texture((png.clone(), readable(TextureFormat::Bgra8Unorm)))
            .await
            .expect("a PNG fills Bgra8Unorm");
        let image = texture.get_image().await.expect("read back");
        assert_eq!(image, blue_first, "create_texture stores blue first");

        let chain = Mipmap::build((png.as_slice(), TextureFormat::Bgra)).expect("Mipmap::build");
        assert_eq!(
            chain.level(0).expect("level 0"),
            blue_first,
            "Mipmap::build"
        );

        let raw = Mipmap::build((&blue_first[..], TextureFormat::Bgra, [1, 1])).expect("raw");
        assert_eq!(
            raw.level(0).expect("level 0"),
            blue_first,
            "raw bytes stay as given"
        );
    });
}

// Story: raw bytes are exactly the size their region takes, so a buffer for a
// larger image, or a 3D size for a mip chain, is refused instead of uploading
// cropped, where only a short buffer was refused.
#[test]
fn raw_bytes_match_their_size() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let long = vec![0; 2 * 2 * 4 + 4];
        let result = renderer.create_texture((long.as_slice(), [2, 2])).await;
        assert!(refused(&result), "create_texture with extra bytes");
        let result = renderer
            .create_storage_texture(([2, 2], TextureFormat::Rgba, long.clone()))
            .await;
        assert!(refused(&result), "a storage seed with extra bytes");

        let texture = renderer
            .create_storage_texture(([64, 64], TextureFormat::Rgba))
            .await
            .expect("create_storage_texture");
        let large = vec![0; 128 * 128 * 4];
        let invalid =
            |result: Result<(), TextureError>| matches!(result, Err(TextureError::InvalidInput(_)));
        assert!(invalid(texture.write(&large)), "write with a larger image");
        let frame = vec![0; 64 * 64 * 4];
        assert!(
            invalid(texture.write_region(&frame, [0, 0, 32, 32])),
            "a whole frame for a smaller region without a stride"
        );
        texture
            .write_region(&frame[..32 * 32 * 4], [0, 0, 32, 32])
            .expect("tight rows for the region");

        let pixels = vec![0; 4 * 4 * 4 * 3];
        let deep = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba, [4, 4, 3]));
        assert!(
            matches!(deep, Err(TextureError::InvalidInput(_))),
            "a 3D chain"
        );
        let longer = Mipmap::build((pixels.as_slice(), TextureFormat::Rgba, [4, 4]));
        assert!(
            matches!(longer, Err(TextureError::InvalidInput(_))),
            "extra bytes"
        );
    });
}

/// Draws the 1x1 mip level of `tex` over the whole target.
const SMALLEST_LEVEL: &str = "
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>, 3>(vec2f(-1., -1.), vec2f(3., -1.), vec2f(-1., 3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> {
  return textureSampleLevel(tex, samp, vec2f(0.5, 0.5), 3.0);
}
";

// Story: a whole-texture write into a texture with a mip chain rebuilds the
// chain, so a texture sampled far away shows the new image, and a partial write,
// which would leave the smaller levels stale, is refused.
#[test]
fn writes_keep_the_mip_chain_current() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let red: Vec<u8> = [255, 0, 0, 255].repeat(8 * 8);
        let texture = renderer
            .create_texture((red.as_slice(), [8, 8]))
            .await
            .expect("a raw texture with a mip chain");
        let green: Vec<u8> = [0, 255, 0, 255].repeat(8 * 8);
        texture.write(&green).expect("a whole write");

        let shader = fragmentcolor::Shader::new(SMALLEST_LEVEL).expect("shader");
        shader.set("tex", &texture).expect("set tex");
        let target = renderer
            .create_texture_target([1, 1])
            .await
            .expect("target");
        renderer.render(&shader, &target).expect("render");
        let image = fragmentcolor::Target::get_image(&target)
            .await
            .expect("read back");
        assert_eq!(
            image,
            [0, 255, 0, 255],
            "the smallest level follows the write"
        );

        let result = texture.write_region(&green[..4 * 4 * 4], [0, 0, 4, 4]);
        assert!(matches!(result, Err(TextureError::InvalidInput(_))));
    });
}

// Story: a size describes raw pixel bytes, so a path or URL with a size is
// refused before any file read or fetch, where the path dropped the size and
// the URL uploaded its encoded bytes as pixels.
#[test]
fn sizes_come_with_raw_bytes_only() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let result = renderer
            .create_texture(("examples/assets/logo.png", [64, 64]))
            .await;
        assert!(refused(&result), "a path with a size");
        let result = renderer
            .create_texture(("https://invalid.invalid/x.png", [64, 64]))
            .await;
        assert!(
            refused(&result),
            "a URL with a size, refused before the fetch"
        );
        let chain = Mipmap::build(("examples/assets/logo.png", [64, 64]));
        assert!(
            matches!(chain, Err(TextureError::InvalidInput(_))),
            "Mipmap::build with a path and a size"
        );
        renderer
            .create_texture("examples/assets/logo.png")
            .await
            .expect("a path without a size decodes");
    });
}

// Story: Mipmap::build reads a file's format from its bytes, as create_texture
// does, so a PNG saved without an extension builds a chain, where the extension
// decided the format and the file failed to decode.
#[test]
fn mip_chains_read_files_like_create_texture() {
    pollster::block_on(async move {
        let folder = tempfile::tempdir().expect("temp folder");
        let path = folder.path().join("3f9a2c");
        let pixel = image::RgbaImage::from_raw(2, 2, vec![255; 16]).expect("pixels");
        std::fs::write(&path, encode(pixel.into(), image::ImageFormat::Png)).expect("write");

        let chain = Mipmap::build((path.as_path(), TextureFormat::Rgba8Unorm))
            .expect("Mipmap::build reads a PNG without an extension");
        assert_eq!(chain.size(), Size::from([2, 2]));
        Renderer::new()
            .create_texture(path.as_path())
            .await
            .expect("create_texture reads it too");
    });
}

// Story: a region with a width, height or depth of 0 is refused, where a zero
// stretched the region to the edge of the texture. Leaving the region out,
// as write does, still covers the whole texture.
#[test]
fn zero_region_sides_are_refused() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let texture = renderer
            .create_storage_texture(([16, 16], TextureFormat::Rgba))
            .await
            .expect("create_storage_texture");
        let rows = vec![0; 16 * 4 * 4];
        let invalid =
            |result: Result<(), TextureError>| matches!(result, Err(TextureError::InvalidInput(_)));
        assert!(
            invalid(texture.write_region(&rows, [0, 0, 0, 4])),
            "zero width"
        );
        assert!(
            invalid(texture.write_region(&rows, [0, 0, 16, 0])),
            "zero height"
        );
        let deep = [0, 0, 0, 16, 4, 0];
        assert!(invalid(texture.write_region(&rows, deep)), "zero depth");
        texture
            .write_region(&rows, [0, 0, 16, 4])
            .expect("a region with every side");
        texture
            .write(&vec![0; 16 * 16 * 4])
            .expect("the whole texture");
    });
}

// Story: as in Swift's and Kotlin's `writeRegion`,
// a `ScreenRegion` whose max lies before its min fails the write
// with an error that names the reversed bounds as the caller wrote them.
#[test]
fn reversed_screen_regions_are_refused() {
    use fragmentcolor::ScreenRegion;
    use fragmentcolor::ScreenRegionError;
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let texture = renderer
            .create_storage_texture(([16, 16], TextureFormat::Rgba))
            .await
            .expect("create_storage_texture");
        let reversed = ScreenRegion {
            min_x: 8,
            min_y: 0,
            max_x: 2,
            max_y: 4,
        };

        let result = texture.write_region(&[0; 6 * 4 * 4], reversed);

        let Err(error) = result else {
            panic!("expected an error, got {result:?}");
        };
        assert!(
            matches!(
                error,
                TextureError::Region(ScreenRegionError::Reversed { .. })
            ),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            "a region needs each max past its min, got x from 8 to 2 and y from 0 to 4"
        );
        let empty = ScreenRegion {
            min_x: 2,
            min_y: 0,
            max_x: 2,
            max_y: 4,
        };
        let result = texture.write_region(&[0; 4 * 4], empty);
        assert!(
            matches!(
                result,
                Err(TextureError::Region(ScreenRegionError::Empty { .. }))
            ),
            "{result:?}"
        );
        let region = ScreenRegion::new((2, 0), (6, 4)).expect("a region");
        texture
            .write_region(&[0; 6 * 4 * 4], region)
            .expect("a checked region");
    });
}

// Story: a buffer copy can't fill a depth texture, so raw pixels, seed bytes
// and a write into Depth32Float are refused before they reach wgpu.
#[test]
fn depth_textures_take_no_uploads() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let bytes = vec![0; 4 * 4 * 4];
        let result = renderer
            .create_texture((bytes.clone(), TextureFormat::Depth32Float, [4, 4]))
            .await;
        assert!(refused(&result), "raw pixels into a depth texture");

        let usage = (TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST).bits;
        let mut seeded = storage_with_usage(TextureFormat::Depth32Float, Some(usage));
        seeded.data = bytes.clone().into();
        let result = renderer.create_storage_texture(seeded).await;
        assert!(refused(&result), "seed bytes into a depth texture");

        let input = storage_with_usage(TextureFormat::Depth32Float, Some(usage));
        let depth = renderer
            .create_storage_texture(input)
            .await
            .expect("an empty depth texture with COPY_DST is made");
        assert!(matches!(
            depth.write(&bytes),
            Err(TextureError::InvalidInput(_))
        ));
    });
}
