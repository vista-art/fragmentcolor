# TextureMipChain

A pre-computed CPU mipmap chain ready to upload as a [Texture](https://fragmentcolor.org/api/core/texture). Build one off the renderer thread (worker / thread pool / async task / Web Worker) so the GPU thread only has to do `queue.write_texture` calls.

You usually do **not** need to construct one of these — `Renderer::create_texture(bytes)` already runs decode + mipmap generation on a background worker on every native target. Reach for `TextureMipChain` when you want explicit control:

- share one chain across many textures without rebuilding it
- fold mipmap generation into an existing decode pipeline that already runs on a worker
- drive prep on your own thread pool (rayon / Swift `Task` / Kotlin `Dispatchers.Default` / Python `ThreadPoolExecutor` / Web Worker)

Construct via the single `TextureMipChain::prepare(input)` entry — same `TextureInput` transport as `Renderer::create_texture` and `Renderer::create_storage_texture`. The discriminator is whether `options.size` is present:
- `prepare((bytes, format))` — encoded image bytes (PNG / JPEG / etc.); size inferred from the decoded image.
- `prepare((bytes, format, size))` — raw pixel bytes already laid out for `format` at `size`.

Cross-language bindings call `prepare(bytes, format, size?)` with `size` optional / nullable.

Consume via the single `Renderer::create_texture(input)` entry — `From<TextureMipChain> for TextureInput` lets you pass the chain directly: `renderer.create_texture(chain).await`. Cross-language users see the same shape (the chain handle goes straight into `createTexture`).

Supported formats are the ones with a CPU mipmap path: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`. Other formats return `TextureError::UnsupportedMipmapFormat`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureMipChain};

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
let chain = TextureMipChain::prepare((png, TextureFormat::Rgba8UnormSrgb))?;

// Hand the chain to the unified create_texture entry - same vocabulary as
// every other texture path; From<TextureMipChain> selects the GPU-only
// upload internally.
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
