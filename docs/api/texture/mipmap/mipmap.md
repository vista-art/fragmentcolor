# Mipmap

A pre-computed CPU mipmap chain ready to upload as a [Texture](https://fragmentcolor.org/api/texture/texture). Build one off the renderer thread (worker, thread pool, async task, Web Worker) so the GPU thread only has to do `queue.write_texture` calls.

Most callers never need to construct this directly: `Renderer::create_texture(bytes)` already runs decode and mipmap generation on a background worker on every native target. Reach for `Mipmap` when you want explicit control:

- share one chain across many textures without rebuilding it,
- bake mipmap generation into a decode pipeline that already runs on a worker,
- drive prep on your own thread pool (rayon, Swift `Task`, Kotlin `Dispatchers.Default`, Python `ThreadPoolExecutor`, Web Worker).

Build with `Mipmap::build(input)`; the input shapes match `Renderer::create_texture` and `Renderer::create_storage_texture`. Whether the bytes are decoded depends on `size`:

- `build((bytes, format))` — encoded image bytes (PNG, JPEG, etc.); size is inferred from the decoded image.
- `build((bytes, format, size))` — raw pixel bytes already laid out for `format` at `size`.

In Swift, Kotlin, JS, and Python, the binding is `build(bytes, format, size?)` with `size` optional.

Upload by passing the chain to `Renderer::create_texture(chain)`; cross-language users hand the chain handle directly to `createTexture`.

Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Other formats return `TextureError::UnsupportedMipmapFormat`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Mipmap, Renderer, TextureFormat};

let renderer = Renderer::new();
// Encoded image bytes the caller has on hand (could come off a worker).
let png: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    // ... full PNG body ...
# // Hidden: build a real 1×1 PNG so the doctest doesn't need a fixture.
];
# let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_vec(1, 1, vec![255, 0, 0, 255]).unwrap());
# let mut png_buf = Vec::new();
# img.write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)?;
# let png = png_buf.as_slice();
let chain = Mipmap::build((png, TextureFormat::Rgba8UnormSrgb))?;

// Upload the chain through the regular create_texture entry point.
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
