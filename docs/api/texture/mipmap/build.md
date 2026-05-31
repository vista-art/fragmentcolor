# Mipmap::build

Build a mip chain on the calling thread, then hand it to [Renderer::create_texture](https://fragmentcolor.org/api/core/renderer/create_texture) for a GPU-only upload. The work is pure CPU (decode if needed, then downsample to 1x1), so call it from wherever you already do heavy work: a worker thread, a thread pool, an async task, a Web Worker. The renderer thread sees only the finished chain.

`build(input)` takes the same input shapes as `Renderer::create_texture` and `Renderer::create_storage_texture`, so you write the call the same way regardless of which entry point you target. Whether `build` decodes the bytes depends on whether you pass a `size`:

- **Encoded path, `size` absent.** The bytes get decoded as PNG / JPEG / BMP / HDR / etc., and the chain inherits the image's dimensions. Use this when you have an encoded blob from disk, the network, or a decoder that emits encoded data.
- **Raw path, `size` present.** The bytes are treated as already laid out for `format` at `size`. Use this when your decoder already produced pixel-format-matching bytes (typical for tile-cache pipelines that fold mip generation into the same step that produced the tile).

`build` runs synchronously and accepts only sync-friendly inputs: bytes, a `DynamicImage`, or a file path. URL inputs (which need an async fetch), KTX2 inputs (already pre-baked), an existing chain, an existing texture, and empty inputs all return `TextureError::InvalidInput` with a message pointing at the right entry point.

Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Anything else returns `TextureError::UnsupportedMipmapFormat`. Decode failures surface as `MalformedImageError`; size and byte-count mismatches as `InvalidInput`.

## Per-language shapes

- **Rust:** `Mipmap::build((bytes, format))` for the encoded path, `Mipmap::build((bytes, format, size))` for the raw path. Tuples implement `Into<TextureInput>` via the `From<T>` impls.
- **JS / Python:** `Mipmap.build(bytes, format, size?)` with `size` optional.
- **Swift / Kotlin:** `Mipmap.build(bytes: ..., format: ...)` for the encoded path, `Mipmap.build(bytes: ..., format: ..., size: ...)` for the raw path. The bindings add overloads so you stick to native positional syntax; wrapping the call in a Rust-style tuple crashes the Swift type checker.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mipmap, Renderer, TextureFormat};

# // Hidden: synthesize a real 1x1 PNG so the doctest doesn't need a fixture.
# let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(1, 1, vec![255, 0, 0, 255]).unwrap());
# let mut png_buf = Vec::new();
# img.write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)?;
# let encoded_png_bytes = png_buf.as_slice();

// Encoded path: bytes plus the format you want the chain to live in.
// The dimensions come from the decoded image.
let chain = Mipmap::build((encoded_png_bytes, TextureFormat::Rgba8UnormSrgb))?;

# let raw_rgba = vec![200u8; 8 * 8 * 4];
// Raw path: include the size so build skips decoding.
let chain_raw = Mipmap::build((
    raw_rgba.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8, 8],
))?;

// Either chain uploads the same way.
let renderer = Renderer::new();
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# _ = chain_raw.count();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
