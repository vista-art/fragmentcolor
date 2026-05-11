# Mipmap::build

Build a mip chain off the renderer thread, then hand it to `Renderer::create_texture` for a GPU-only upload. The work is pure CPU; call it from any worker, thread pool, async task, or Web Worker.

`build(input)` accepts the same shapes as `Renderer::create_texture` and `Renderer::create_storage_texture`. Whether the bytes are decoded depends on `size`:

- **Encoded:** `size` absent. The bytes are decoded as PNG / JPEG / etc. and the chain inherits the image's dimensions. Use this when you have an encoded blob from disk, the network, or a decoder that emits encoded data.
- **Raw:** `size` present. The bytes are treated as already laid out for `format` at `size`. Use this when your decoder already produced pixel-format-matching bytes (typical for tile-cache pipelines that bake mipmap generation into the same step).

`build` runs synchronously and accepts only sync-friendly inputs: bytes, a `DynamicImage`, or a file path. URL inputs (which need async fetch), KTX2 inputs (already pre-baked), an existing chain, an existing texture, and empty inputs all return `TextureError::InvalidInput` with a message pointing at the right entry point.

Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Other formats return `TextureError::UnsupportedMipmapFormat`. Decode failures surface as `MalformedImageError`; size and byte-count mismatches as `InvalidInput`.

The cross-language bindings expose `build(bytes, format, size?)` with `size` optional. Swift and Kotlin add overloads so you call `Mipmap.build(bytes:format:)` for the encoded path and `Mipmap.build(bytes:format:size:)` for the raw path.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mipmap, Renderer, TextureFormat};

# // Build a real 1×1 PNG so the doctest doesn't need a fixture.
# let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(1, 1, vec![255, 0, 0, 255]).unwrap());
# let mut png_buf = Vec::new();
# img.write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)?;
# let encoded_png_bytes = png_buf.as_slice();
// Encoded path: pass bytes plus the format you expect.
let chain = Mipmap::build((encoded_png_bytes, TextureFormat::Rgba8UnormSrgb))?;

# let raw_rgba = vec![200u8; 8 * 8 * 4];
// Raw path: include the size so build skips decoding.
let chain_raw = Mipmap::build((
    raw_rgba.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8, 8],
))?;

// Upload the chain through the regular create_texture entry point.
let renderer = Renderer::new();
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# _ = chain_raw.count();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
