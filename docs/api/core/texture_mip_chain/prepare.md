# TextureMipChain::prepare

Build a mip chain off the renderer thread, then hand it to `Renderer::create_texture` for a GPU-only upload. Pure CPU work — call from any worker / thread pool / async task / Web Worker.

`prepare(input)` takes the same `TextureInput` transport as `Renderer::create_texture` and `Renderer::create_storage_texture` — one vocabulary across the API. The discriminator is whether `options.size` is set:

- **Encoded path** — `size` absent. `data: Bytes` is decoded internally and the chain inherits the image's dimensions. Use when you have PNG / JPEG bytes from disk, network, or a worker decoder that emits encoded blobs.
- **Raw path** — `size` present. `data: Bytes` is treated as already laid out for `format` at `size` — no decode. Use when your decoder already produced pixel-format-matching bytes (typical for tile-cache pipelines that fold mipmap generation into the same hop).

`prepare` requires a sync-friendly `data` variant: `Bytes`, `DynamicImage`, or `Path` (file IO). `Url` (needs async fetch), `Ktx2*` (already pre-baked), `Prepared` (already a chain), `CloneOf` (existing texture), and `Empty` (no source) all return `TextureError::InvalidInput` with a clear message pointing at the right entry point.

Supported formats for the mipmap chain: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Other formats return `TextureError::UnsupportedMipmapFormat`. Decode failures surface as `MalformedImageError`; size / byte-count mismatches as `InvalidInput`.

Rust callers use the `From<T>` impls — there's almost never a reason to construct `TextureInput` literally:

| Form | Produces | Path |
|------|----------|------|
| `(bytes, format)` | `data: Bytes`, `options.format = format` | encoded |
| `(bytes, format, [w, h])` | adds `options.size = Some([w, h])` | raw |
| `(bytes, format, (w, h))` | same as above | raw |
| `(bytes, format, Size::from(...))` | same as above | raw |

Cross-language bindings expose `prepare(bytes, format, size?)` — `size` is optional / nullable. Swift / Kotlin extensions wrap the constructor so users call `TextureMipChain.prepare(bytes:format:)` (encoded) or `TextureMipChain.prepare(bytes:format:size:)` (raw) without seeing the underlying `TextureInput` plumbing.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureMipChain};

# // Build a real 1×1 PNG so the doctest doesn't need a fixture.
# let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(1, 1, vec![255, 0, 0, 255]).unwrap());
# let mut png_buf = Vec::new();
# img.write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)?;
# let encoded_png_bytes = png_buf.as_slice();
// Encoded path — single tuple, no extra method.
let chain = TextureMipChain::prepare((encoded_png_bytes, TextureFormat::Rgba8UnormSrgb))?;

# let raw_rgba = vec![200u8; 8 * 8 * 4];
// Raw pixel path — same method, just include the size in the tuple.
let chain_raw = TextureMipChain::prepare((
    raw_rgba.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [8u32, 8u32],
))?;

// Hand the chain to the unified create_texture entry — same vocabulary.
let renderer = Renderer::new();
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# _ = chain_raw.level_count();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
