//! The texture tests: decoding, conversions, mip chains, float levels,
//! the format checks and the input `create_texture` refuses.

use super::loading::decode::{array_pixels, check_pixels, decode, decode_pixels};
use super::mipmap::{halve, read_floats};
use super::*;
use image::DynamicImage;

/// Encodes `image` as PNG bytes, the way a file on disk would hold it.
fn encode_png(image: &DynamicImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encode png");
    bytes
}

// Story: a PNG with corrupt pixel data
// names the decoder's reason exactly once in the error.
#[test]
fn malformed_images_name_their_reason_once() {
    let mut bytes = encode_png(&DynamicImage::new_rgba8(1, 1));
    let data = bytes
        .windows(4)
        .position(|window| window == b"IDAT")
        .map(|start| start + 4)
        .expect("an IDAT chunk");
    bytes[data] ^= 0xFF;
    let reason = decode_pixels(&bytes)
        .err()
        .and_then(|failure| std::error::Error::source(&failure.0).map(ToString::to_string))
        .expect("a decoder reason");

    let error = decode(&bytes, 64).err().map(|error| error.to_string());

    let text = error.expect("a decode error");
    assert!(text.starts_with("Image decode failed: "), "{text}");
    assert_eq!(text.matches(reason.as_str()).count(), 1, "{text}");
}

// Story: TextureInput conversions from common Rust types produce expected variants.
#[test]
fn texture_input_conversions() {
    let bytes: Vec<u8> = vec![1, 2, 3];
    let ti1: TextureData = (&bytes).into();
    match ti1 {
        TextureData::Bytes(b) => assert_eq!(b, bytes),
        _ => panic!("expected Bytes"),
    }

    let p = std::path::PathBuf::from("/tmp/img.png");
    let ti2: TextureData = (&p).into();
    match ti2 {
        TextureData::Path(pb) => assert_eq!(pb, p),
        _ => panic!("expected Path"),
    }
}

// Story: a string is a URL with an http(s) scheme and a file path otherwise.
#[test]
fn texture_input_from_strings() {
    assert!(matches!(
        TextureData::from("https://example.com/logo.png"),
        TextureData::Url(url) if url == "https://example.com/logo.png"
    ));
    assert!(matches!(
        TextureData::from(String::from("assets/logo.png")),
        TextureData::Path(path) if path == std::path::Path::new("assets/logo.png")
    ));
    let input = TextureInput::from(("logo.png", [2u32, 2]));
    assert!(matches!(input.data, TextureData::Path(_)));
    assert_eq!(input.options.size, Some(Size::from([2u32, 2])));
}

// Story: Mipmap::level returns one level and errors past the end.
#[test]
fn mipmap_level_indexes_the_chain() {
    let pixels = vec![9u8; 4 * 4 * 4];
    let chain = Mipmap::build((
        pixels.as_slice(),
        crate::TextureFormat::Rgba8Unorm,
        [4u32, 4],
    ))
    .expect("chain");
    assert_eq!(chain.level(0).expect("level 0").len(), 4 * 4 * 4);
    assert_eq!(chain.level(2).expect("level 2").len(), 4);
    assert!(matches!(chain.level(3), Err(TextureError::InvalidInput(_))));
}

// Story: an ndarray's bytes pass as they are when its channels match the
// format, gain alpha for an 8-bit RGBA format, and anything else, a shape
// past u32 included, is refused before its bytes are read.
#[test]
fn array_pixels_fit_the_format() {
    let gray = [1u8, 2, 3, 4];
    let read = || gray.to_vec();
    let (bytes, size) =
        array_pixels(&[2, 2], read, wgpu::TextureFormat::R8Unorm).expect("gray as R8");
    assert_eq!((bytes, size), (gray.to_vec(), Size::new(2, 2, None)));
    let (bytes, _) =
        array_pixels(&[2, 2], read, wgpu::TextureFormat::Rgba8Unorm).expect("gray as RGBA");
    assert_eq!(&bytes[..8], &[1, 1, 1, 255, 2, 2, 2, 255]);
    let (bytes, size) =
        array_pixels(&[1, 2, 2], read, wgpu::TextureFormat::Rg8Unorm).expect("two channels as Rg8");
    assert_eq!((bytes, size), (gray.to_vec(), Size::new(2, 1, None)));

    let unread = || -> Vec<u8> { panic!("the bytes of a refused shape are never read") };
    for (shape, format) in [
        (vec![2, 2], wgpu::TextureFormat::Rgba32Float),
        (vec![2, 2, 3], wgpu::TextureFormat::Rg16Unorm),
        (vec![2, 2, 5], wgpu::TextureFormat::Rgba8Unorm),
        (vec![2, 2, 2, 2], wgpu::TextureFormat::Rgba8Unorm),
        (vec![1, (1 << 32) + 4, 3], wgpu::TextureFormat::Rgba8Unorm),
    ] {
        assert!(
            matches!(
                array_pixels(&shape, unread, format),
                Err(TextureError::InvalidInput(_))
            ),
            "{shape:?} as {format:?}"
        );
    }
}

// Story: options for a source with its own RGBA8 pixels keep its size and an
// 8-bit RGBA or BGRA format, and anything else is refused.
#[test]
fn check_pixels_keeps_the_source_layout() {
    let own = Size::new(2, 1, None);
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
    ] {
        check_pixels(own, None, format).expect("an RGBA8 format");
    }
    check_pixels(
        own,
        Some(Size::new(2, 1, Some(1))),
        wgpu::TextureFormat::Rgba8Unorm,
    )
    .expect("the source's own size");
    for (size, format) in [
        (Some(Size::new(1, 1, None)), wgpu::TextureFormat::Rgba8Unorm),
        (None, wgpu::TextureFormat::R8Unorm),
        (None, wgpu::TextureFormat::Rgba32Float),
    ] {
        assert!(
            matches!(
                check_pixels(own, size, format),
                Err(TextureError::InvalidInput(_))
            ),
            "{size:?} {format:?}"
        );
    }
}

// Story: Create a texture from encoded image bytes and verify aspect.
#[test]
fn encoded_image_aspect_smoke() {
    pollster::block_on(async move {
        // 2x1 image in memory
        let img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_vec(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]).unwrap(),
        );

        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Bytes(encode_png(&img)))
            .await
            .expect("create texture from encoded image");
        assert_eq!(tex.aspect(), 2.0);

        // Change sampler options should not panic
        tex.set_sampler_options(SamplerOptions {
            repeat_x: false,
            repeat_y: false,
            smooth: true,
            compare: None,
        });
    });
}

// Story: mip_level_count_for matches the standard "down to 1×1" chain length.
#[test]
fn mip_level_count_for_known_dims() {
    assert_eq!(mip_level_count_for(0, 0), 1);
    assert_eq!(mip_level_count_for(1, 1), 1);
    assert_eq!(mip_level_count_for(2, 1), 2);
    assert_eq!(mip_level_count_for(1, 2), 2);
    assert_eq!(mip_level_count_for(4, 4), 3); // 4, 2, 1
    assert_eq!(mip_level_count_for(1024, 512), 11); // 1024 → 1
    // Non-power-of-two: chain length follows the largest dim.
    assert_eq!(mip_level_count_for(1000, 600), 10); // floor(log2(1000)) + 1
}

// Story: Default options request a mipmap chain; explicit opt-out yields a single level.
#[test]
fn encoded_image_creates_mipmap_chain_by_default() {
    pollster::block_on(async move {
        // 8x8 solid color image — generates 4 mip levels (8, 4, 2, 1).
        let pixels = vec![200u8; 8 * 8 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
        let png = encode_png(&img);
        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Bytes(png.clone()))
            .await
            .expect("create texture with mipmaps");
        assert_eq!(tex.object().inner.mip_level_count(), 4);

        // Opt-out: only mip 0
        let opts = TextureOptions {
            mipmaps: false,
            ..Default::default()
        };
        let tex_no_mip = r
            .create_texture((TextureData::Bytes(png), opts))
            .await
            .expect("create texture without mipmaps");
        assert_eq!(tex_no_mip.object().inner.mip_level_count(), 1);
    });
}

// Story: Format override is honored on the encoded image arm — same RGBA bytes,
// reinterpreted as linear instead of sRGB. Mipmap chain is still generated.
#[test]
fn format_override_honored_for_image_input() {
    pollster::block_on(async move {
        let pixels = vec![128u8; 4 * 4 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(4, 4, pixels).unwrap());
        let r = crate::Renderer::new();

        let opts = TextureOptions {
            format: crate::TextureFormat::Rgba8Unorm,
            ..Default::default()
        };
        let tex = r
            .create_texture((TextureData::Bytes(encode_png(&img)), opts))
            .await
            .expect("create texture with explicit linear format");
        assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
    });
}

// Story: 16-bit grayscale source (e.g. height-map PNG) is decoded with
// to_luma16 — preserving precision — and uploaded as R16Unorm with mips.
#[test]
fn luma16_image_uploads_as_r16unorm_with_mipmaps() {
    pollster::block_on(async move {
        // 4×4 ramp where each pixel needs the full u16 range to round-trip.
        let pixels: Vec<u16> = (0..16).map(|i| i * 4096).collect();
        let buf = image::ImageBuffer::<image::Luma<u16>, _>::from_vec(4, 4, pixels).unwrap();
        let img = image::DynamicImage::ImageLuma16(buf);

        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Bytes(encode_png(&img)))
            .await
            .expect("create R16Unorm texture from Luma16 image");
        assert_eq!(tex.object().format, wgpu::TextureFormat::R16Unorm);
        assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
    });
}

/// The texels of mip `level` of `texture`, copied straight from the GPU.
fn read_level(texture: &Texture, level: u32) -> Vec<u8> {
    let object = texture.object();
    let context = &texture.context;
    let bpp = measure_texel(object.format).expect("texel size");
    let (width, height) = (
        (object.size.width >> level).max(1),
        (object.size.height >> level).max(1),
    );
    let (row, padded) = (width * bpp, (width * bpp).div_ceil(256) * 256);
    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Level readback"),
        size: u64::from(padded * height),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = context.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &object.inner,
            mip_level: level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    context.queue.submit(Some(encoder.finish()));
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .expect("poll");
    let view = slice.get_mapped_range().expect("mapped range");
    let rows = view.chunks(padded as usize).take(height as usize);
    rows.flat_map(|line| line[..row as usize].to_vec())
        .collect()
}

/// The bytes of `texels` in the float `format`.
fn encode_floats(texels: &[f32], format: crate::TextureFormat) -> Vec<u8> {
    match format {
        crate::TextureFormat::Rgba16Float => texels
            .iter()
            .flat_map(|value| half::f16::from_f32(*value).to_le_bytes())
            .collect(),
        _ => texels
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
    }
}

/// The first texel of `bytes`, read as the float `format`.
fn read_texel(bytes: &[u8], format: crate::TextureFormat) -> Vec<f32> {
    read_floats(bytes, format.to_wgpu())[..4].to_vec()
}

// Story: a float mip level keeps values past 0 to 1, from raw bytes, a whole
// write, Mipmap::build and an HDR decode.
#[test]
fn float_levels_keep_their_range() {
    pollster::block_on(async move {
        let r = crate::Renderer::new();
        let sky = [10.0, -2.0, 0.5, 1.0];
        let night = [20.0, -4.0, 0.25, 1.0];
        for format in [
            crate::TextureFormat::Rgba16Float,
            crate::TextureFormat::Rgba32Float,
        ] {
            let bytes = encode_floats(&sky.repeat(4), format);
            let options = TextureOptions {
                size: Some([2u32, 2].into()),
                format,
                ..Default::default()
            }
            .with_usage(crate::TextureUsage::COPY_SRC);
            let tex = r
                .create_texture((bytes.clone(), options))
                .await
                .expect("raw float texture");
            assert_eq!(tex.object().inner.mip_level_count(), 2);
            assert_eq!(read_texel(&read_level(&tex, 1), format), sky, "{format:?}");

            tex.write(&encode_floats(&night.repeat(4), format))
                .expect("a whole write");
            let level = read_level(&tex, 1);
            assert_eq!(read_texel(&level, format), night, "{format:?} write");

            let chain = Mipmap::build((bytes.as_slice(), format, [2u32, 2])).expect("chain");
            let level = chain.level(1).expect("level 1");
            assert_eq!(read_texel(level, format), sky, "{format:?} Mipmap");
        }

        let hdr_sky = image::Rgb32FImage::from_pixel(2, 2, image::Rgb([10.0, 5.0, 2.5]));
        let mut hdr = Vec::new();
        DynamicImage::ImageRgb32F(hdr_sky)
            .write_to(&mut std::io::Cursor::new(&mut hdr), image::ImageFormat::Hdr)
            .expect("encode hdr");
        let options = TextureOptions::default().with_usage(crate::TextureUsage::COPY_SRC);
        let tex = r
            .create_texture((TextureData::Bytes(hdr), options))
            .await
            .expect("an HDR file");
        let format = crate::TextureFormat::Rgba16Float;
        let texel = read_texel(&read_level(&tex, 1), format);
        assert!(
            (texel[0] - 10.0).abs() < 0.2,
            "HDR level 1 keeps 10, got {texel:?}"
        );
    });
}

// Story: raw half floats upload as given at level 0, an infinity and a NaN
// payload included, and only the levels below it are resampled.
#[test]
fn raw_half_floats_keep_level_zero() {
    pollster::block_on(async move {
        let words: [u16; 16] = [0x7C00, 0x7E01, 0x3C00, 0x3C00]
            .repeat(4)
            .try_into()
            .unwrap();
        let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        let format = crate::TextureFormat::Rgba16Float;
        let options = TextureOptions {
            size: Some([2u32, 2].into()),
            format,
            ..Default::default()
        }
        .with_usage(crate::TextureUsage::COPY_SRC);
        let r = crate::Renderer::new();
        let tex = r
            .create_texture((bytes.clone(), options))
            .await
            .expect("raw half floats");
        assert_eq!(tex.object().inner.mip_level_count(), 2);
        assert_eq!(read_level(&tex, 0), bytes, "create_texture keeps level 0");
        tex.write(&bytes).expect("a whole write");
        assert_eq!(read_level(&tex, 0), bytes, "write keeps level 0");
        let chain = Mipmap::build((bytes.as_slice(), format, [2u32, 2])).expect("chain");
        assert_eq!(
            chain.level(0).expect("level 0"),
            bytes,
            "Mipmap keeps level 0"
        );
    });
}

// Story: on an odd side the last texel joins the span before it,
// so every texel of a 5x3 or a 3x1 image counts once in the level below.
#[test]
fn halve_averages_odd_sides() {
    let gradient = |width, height| {
        image::Rgba32FImage::from_fn(width, height, |column, row| {
            image::Rgba([(column + 10 * row) as f32; 4])
        })
    };
    let level = halve(&gradient(5, 3));
    assert_eq!(level.dimensions(), (2, 1));
    assert_eq!(level.get_pixel(0, 0).0, [10.5; 4], "6 texels on the left");
    assert_eq!(level.get_pixel(1, 0).0, [13.0; 4], "9 texels on the right");
    let level = halve(&gradient(3, 1));
    assert_eq!(level.dimensions(), (1, 1));
    assert_eq!(level.get_pixel(0, 0).0, [1.0; 4], "all 3 texels");
}

// Story: an HDR file and a float format get the full mip chain every other
// source image gets by default, in create_texture and in Mipmap::build.
#[test]
fn float_images_get_a_mip_chain() {
    pollster::block_on(async move {
        let sky = image::Rgb32FImage::from_pixel(8, 8, image::Rgb([2.0, 1.0, 0.5]));
        let mut hdr = Vec::new();
        DynamicImage::ImageRgb32F(sky)
            .write_to(&mut std::io::Cursor::new(&mut hdr), image::ImageFormat::Hdr)
            .expect("encode hdr");
        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Bytes(hdr))
            .await
            .expect("create texture from an HDR file");
        assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba16Float);
        assert_eq!(tex.object().inner.mip_level_count(), 4); // 8, 4, 2, 1

        let pixels = vec![128u8; 8 * 8 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
        let png = encode_png(&img);
        let opts = TextureOptions::from(crate::TextureFormat::Rgba32Float);
        let tex = r
            .create_texture((TextureData::Bytes(png.clone()), opts))
            .await
            .expect("create Rgba32Float texture from a PNG");
        assert_eq!(tex.object().inner.mip_level_count(), 4);

        let chain = Mipmap::build((png.as_slice(), crate::TextureFormat::Rgba16Float))
            .expect("a half float chain");
        let sizes: Vec<usize> = chain.levels().iter().map(Vec::len).collect();
        assert_eq!(sizes, [8 * 64, 8 * 16, 8 * 4, 8]);
        let half = half::f16::from_le_bytes([chain.levels()[3][0], chain.levels()[3][1]]);
        assert!(
            (half.to_f32() - 128.0 / 255.0).abs() < 0.01,
            "level 3 keeps the value"
        );
    });
}

// Story: 8-bit grayscale source goes to R8Unorm with the matching CPU mipmap path.
#[test]
fn luma8_image_uploads_as_r8unorm_with_mipmaps() {
    pollster::block_on(async move {
        let pixels: Vec<u8> = (0..16u8).map(|i| i * 16).collect();
        let buf = image::GrayImage::from_vec(4, 4, pixels).unwrap();
        let img = image::DynamicImage::ImageLuma8(buf);

        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Bytes(encode_png(&img)))
            .await
            .expect("create R8Unorm texture from Luma8 image");
        assert_eq!(tex.object().format, wgpu::TextureFormat::R8Unorm);
        assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
    });
}

// Story: prepare with the From<(bytes, format)> impl produces a chain
// whose level count and per-level byte sizes match what Renderer::create_texture
// computes inline. Mirrors the KTX2 "trust the caller's prebaked levels"
// path but for runtime-prepared chains coming off a worker thread.
#[test]
fn prepare_builds_mipmap_chain_matching_inline_path() {
    // 8×8 RGBA8 PNG → 4 mip levels (8, 4, 2, 1).
    let pixels = vec![123u8; 8 * 8 * 4];
    let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    // From<(&[u8], TextureFormat)> — encoded image path.
    let chain =
        Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb)).expect("prepare ok");
    assert_eq!(chain.count(), 4);
    assert_eq!(chain.size(), Size::from([8, 8]));
    assert_eq!(chain.format(), crate::TextureFormat::Rgba8UnormSrgb);
    let expected = [4 * 8 * 8, 4 * 4 * 4, 4 * 2 * 2, 4];
    for (i, level) in chain.levels().iter().enumerate() {
        assert_eq!(level.len(), expected[i], "level {i} byte count");
    }
}

// Story: the raw-pixel path is selected by providing a `size` — same
// single `prepare` entry, the From<(bytes, format, size)> impl carries
// it. Exercises tuple, array, and Size as the size argument.
#[test]
fn prepare_raw_builds_chain_from_raw_pixels() {
    let pixels = vec![200u8; 8 * 8 * 4];
    // From<(&[u8], TextureFormat, (u32, u32))>
    let chain = Mipmap::build((
        pixels.as_slice(),
        crate::TextureFormat::Rgba8UnormSrgb,
        (8u32, 8u32),
    ))
    .expect("prepare with tuple size");
    assert_eq!(chain.count(), 4);
    assert_eq!(chain.size(), Size::from([8, 8]));

    // From<(&[u8], TextureFormat, [u32; 2])>
    let chain2 = Mipmap::build((
        pixels.as_slice(),
        crate::TextureFormat::Rgba8UnormSrgb,
        [8u32, 8u32],
    ))
    .expect("prepare with array size");
    assert_eq!(chain2.size(), Size::from([8, 8]));

    // Explicit `TextureInput` struct literal — same transport as
    // `Renderer::create_texture` and `create_storage_texture`.
    let chain3 = Mipmap::build(crate::TextureInput {
        data: crate::TextureData::Bytes(pixels.clone()),
        options: crate::TextureOptions {
            size: Some(crate::Size::from([8u32, 8u32])),
            format: crate::TextureFormat::Rgba8UnormSrgb,
            ..Default::default()
        },
    })
    .expect("prepare with explicit TextureInput");
    assert_eq!(chain3.size(), Size::from([8, 8]));
}

// Story: prepare's three failure modes (decode, format, shape) surface as
// distinct TextureError variants so callers logging tile-cache errors can
// distinguish "this tile's bytes are corrupt" from "this format isn't
// supported by FC's CPU mipmap path" from "the bytes don't match the
// declared shape" without parsing strings.
#[test]
fn prepare_error_variants_are_distinct() {
    // 1) Decode failure → MalformedImageError (from `image::ImageError`).
    let garbage = vec![0xFFu8; 32];
    let err = Mipmap::build((garbage.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
        .expect_err("garbage bytes should fail to decode");
    assert!(
        matches!(err, TextureError::MalformedImageError(_)),
        "expected MalformedImageError, got {err:?}"
    );

    // 2) Unsupported format → UnsupportedMipmapFormat carrying the
    //    public TextureFormat the caller passed in.
    let pixels = vec![0u8; 4 * 4 * 4];
    let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(4, 4, pixels).unwrap());
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    let err = Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba32Uint))
        .expect_err("Rgba32Uint not supported by CPU mipmap path");
    match err {
        TextureError::UnsupportedMipmapFormat { format } => {
            assert_eq!(format, crate::TextureFormat::Rgba32Uint);
        }
        other => panic!("expected UnsupportedMipmapFormat, got {other:?}"),
    }

    // 3) Shape mismatch (raw path) → InvalidInput.
    let too_few_bytes = vec![0u8; 16];
    let err = Mipmap::build((
        too_few_bytes.as_slice(),
        crate::TextureFormat::Rgba8UnormSrgb,
        (32u32, 32u32),
    ))
    .expect_err("declared 32x32 RGBA but only gave 16 bytes");
    assert!(
        matches!(err, TextureError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );

    // 4) Zero size (raw path) → InvalidInput.
    let err = Mipmap::build((&[][..], crate::TextureFormat::Rgba8UnormSrgb, (0u32, 16u32)))
        .expect_err("zero-width raw input should fail");
    assert!(
        matches!(err, TextureError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
}

// Story: A prepared chain feeds back into create_texture and produces a
// texture with the expected format + mip chain — the round-trip consumers
// use for off-main-thread mip generation. Exercises both the
// TextureInput::Prepared variant (Rust ergonomics) and the dedicated
// create_texture_prepared cross-language entry point.
// Story: raw pixels for a 3D texture keep one mip level, so the upload
// writes every slice instead of resampling the first one.
#[test]
fn raw_3d_texture_keeps_one_level() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let pixels: Vec<u8> = (0..2 * 2 * 3 * 4).map(|value| value as u8).collect();
        let texture = renderer
            .create_texture((pixels.as_slice(), [2, 2, 3]))
            .await
            .expect("create_texture failed");
        let object = texture.object();
        assert_eq!(object.inner.dimension(), wgpu::TextureDimension::D3);
        assert_eq!(object.inner.mip_level_count(), 1);
    });
}

#[test]
fn prepared_input_round_trips_through_create_texture() {
    pollster::block_on(async move {
        let pixels = vec![77u8; 8 * 8 * 4];
        let img =
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(8, 8, pixels).unwrap());
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let chain = Mipmap::build((buf.as_slice(), crate::TextureFormat::Rgba8UnormSrgb))
            .expect("prepare ok");
        let level_count = chain.count() as u32;
        let r = crate::Renderer::new();
        let tex = r
            .create_texture(TextureData::Prepared(chain.clone()))
            .await
            .expect("create_texture from Prepared chain");
        assert_eq!(tex.object().format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(tex.object().inner.mip_level_count(), level_count);
        let sz = tex.size();
        assert_eq!(sz.width, 8);
        assert_eq!(sz.height, 8);

        // create_texture(chain) reaches the same path via
        // From<Mipmap> for TextureInput. No second method needed.
        let tex2 = r
            .create_texture(chain)
            .await
            .expect("create_texture from chain");
        assert_eq!(tex2.object().format, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_eq!(tex2.object().inner.mip_level_count(), level_count);
    });
}

// Story: Raw bytes with R16Unorm format — caller hands us packed little-endian
// u16s, we wrap them as a Luma16 buffer and resample for each mip level.
#[test]
fn raw_bytes_r16unorm_generates_mipmap_chain() {
    pollster::block_on(async move {
        // 4×4 image with each u16 cycling through the full range.
        let words: Vec<u16> = (0..16).map(|i| i * 4096).collect();
        let mut bytes: Vec<u8> = Vec::with_capacity(words.len() * 2);
        for w in &words {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        let opts = TextureOptions {
            size: Some(crate::Size::from([4u32, 4u32])),
            format: crate::TextureFormat::R16Unorm,
            ..Default::default()
        };
        let r = crate::Renderer::new();
        let tex = r
            .create_texture((&bytes[..], opts))
            .await
            .expect("create R16Unorm texture from raw bytes");
        assert_eq!(tex.object().format, wgpu::TextureFormat::R16Unorm);
        assert_eq!(tex.object().inner.mip_level_count(), 3); // 4, 2, 1
    });
}

// Story: a format that needs a wgpu feature the device lacks is refused for
// any usage, with the typed error naming the missing feature. Pure, no GPU.
#[test]
fn check_feature_names_the_missing_feature() {
    let empty = wgpu::Features::empty();
    let norm = wgpu::Features::TEXTURE_FORMAT_16BIT_NORM;
    for format in [
        wgpu::TextureFormat::R16Unorm,
        wgpu::TextureFormat::Rg16Unorm,
        wgpu::TextureFormat::Rgba16Unorm,
    ] {
        match check_feature(empty, format) {
            Err(TextureError::UnsupportedFormatForUsage {
                format: refused,
                missing_feature,
            }) => {
                assert_eq!(refused, format!("{format:?}"));
                assert_eq!(missing_feature, "TEXTURE_FORMAT_16BIT_NORM");
            }
            other => panic!("expected UnsupportedFormatForUsage, got {other:?}"),
        }
        check_feature(norm, format).expect("with the feature");
    }
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba32Float,
        wgpu::TextureFormat::Depth32Float,
    ] {
        check_feature(empty, format).expect("needs no feature");
    }
}

// Story: a usage the format can't take, such as storage on Depth32Float,
// Rgba8UnormSrgb or Bgra8Unorm, and a depth format with layers are refused
// with the error resize returns. Pure, against the WebGPU baseline.
#[test]
fn check_support_refuses_what_the_format_cannot_take() {
    let flat = wgpu::TextureDimension::D2;
    let deep = wgpu::TextureDimension::D3;
    let storage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
    let sampled = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    let check = |format: wgpu::TextureFormat, usage, dimension| {
        let allowed = format
            .guaranteed_format_features(wgpu::Features::empty())
            .allowed_usages;
        check_support(allowed, format, usage, dimension)
    };
    for format in [
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::R8Unorm,
    ] {
        assert!(
            matches!(
                check(format, storage, flat),
                Err(TextureError::InvalidInput(_))
            ),
            "{format:?} takes no storage"
        );
        check(format, sampled, flat).expect("every format samples");
    }
    check(wgpu::TextureFormat::Rgba8Unorm, storage, flat).expect("Rgba8Unorm stores");
    check(wgpu::TextureFormat::Rgba8Unorm, sampled, deep).expect("a color format in 3D");
    assert!(matches!(
        check(wgpu::TextureFormat::Depth32Float, sampled, deep),
        Err(TextureError::InvalidInput(_))
    ));
}

/// Whether `result` failed with `InvalidInput` whose message holds `text`.
fn refused<T: std::fmt::Debug>(result: &Result<T, crate::RendererError>, text: &str) -> bool {
    use crate::RendererError;
    matches!(result, Err(RendererError::TextureError(TextureError::InvalidInput(message))) if message.contains(text))
}

/// A 4x4 chain built from raw pixels in its default format, `Rgba8Unorm`, as an input
/// that asks for `format`.
fn ask_format(format: crate::TextureFormat) -> TextureInput {
    let chain = Mipmap::build((vec![255u8; 4 * 4 * 4], [4u32, 4u32])).expect("a chain");
    TextureInput {
        data: chain.into(),
        options: TextureOptions {
            format,
            ..Default::default()
        },
    }
}

// Story: a Mipmap uploads in the format it was built in, or in that format's sRGB form,
// which reads the same bytes. Any other format given with it is refused.
#[test]
fn a_mipmap_takes_only_a_format_its_bytes_read_as() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();

        let srgb = ask_format(crate::TextureFormat::Rgba8UnormSrgb);
        let texture = renderer.create_texture(srgb).await.expect("the sRGB form");
        assert_eq!(texture.object().format, wgpu::TextureFormat::Rgba8UnormSrgb);

        let own = ask_format(crate::TextureFormat::default());
        let texture = renderer.create_texture(own).await.expect("its own format");
        assert_eq!(texture.object().format, wgpu::TextureFormat::Rgba8Unorm);

        let float = ask_format(crate::TextureFormat::Rgba32Float);
        let result = renderer.create_texture(float).await;
        assert!(refused(&result, "create_texture: a Mipmap"), "{result:?}");
    });
}

// Story: a Texture keeps the options it was created with, so `create_texture` refuses
// options beside one with an error that names the call, and takes the Texture alone as it is.
#[test]
fn create_texture_refuses_options_beside_a_texture() {
    pollster::block_on(async move {
        let renderer = crate::Renderer::new();
        let pixels = (vec![255u8; 4 * 4 * 4], [4u32, 4u32]);
        let texture = renderer.create_texture(pixels).await.expect("a texture");

        let given = (&texture, crate::TextureFormat::Rgba32Float);
        let result = renderer.create_texture(given).await;
        assert!(refused(&result, "create_texture:"), "{result:?}");
        let same = renderer
            .create_texture(&texture)
            .await
            .expect("the Texture alone");
        assert_eq!(same.id(), texture.id());
    });
}
