# Mipmap

A `Mipmap` is a finished CPU mip chain (the base image plus every half-size copy down to 1x1) ready to upload as a [Texture](https://fragmentcolor.org/api/texture/texture). Mipmaps are what lets a sampler pick a smaller, pre-filtered copy when a textured surface shrinks in screen space; without them, minified textures shimmer and alias. FragmentColor generates the chain for you whenever you call `Renderer::create_texture(bytes)`, so you rarely need to build one by hand.

Reach for `Mipmap` when you want to drive the decode and resize work yourself: on a worker thread, in a tile-cache pipeline, or wherever you already own threading and want the renderer thread to do nothing but `queue.write_texture`. That's the whole point of the type: keep CPU prep off the renderer thread, and hand the upload a chain that's already done.

Typical reasons to build one explicitly:

- Your asset pipeline already runs on a worker (rayon, Swift `Task`, Kotlin `Dispatchers.Default`, Python `ThreadPoolExecutor`, a Web Worker) and you want to fold mip generation into the same hop.
- You stream tiles into a paint-canvas or terrain renderer and need each tile's GPU upload to be a single write, not a decode-plus-resize per upload.
- You want to share one chain across many textures without rebuilding it.

Build with [Mipmap::build(input)](https://fragmentcolor.org/api/texture/mipmap#mipmapbuild); the input shapes match what [Renderer::create_texture](https://fragmentcolor.org/api/core/renderer#renderercreate_texture) accepts. Whether the bytes get decoded depends on whether you pass a `size`:

- `build((bytes, format))`: encoded image (PNG, JPEG, BMP, HDR, ...). FC decodes and infers the dimensions.
- `build((bytes, format, size))`: raw pixel bytes already laid out for `format` at `size`. No decode.

Then hand the chain to `Renderer::create_texture(chain)` for the upload. The cross-language bindings expose `build(bytes, format, size?)` with `size` optional.

Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`, and the `Rgba`, `Bgra`, `L8` and `Lab` aliases. Anything else returns `TextureError::UnsupportedMipmapFormat`.

## Per-language shapes

- **Rust:** `Mipmap::build(input)` where `input` is `(bytes, format)`, `(bytes, format, size)`, or a file path.
- **JS:** `Mipmap.build(bytes, format, size?)` with `bytes` as `Uint8Array` / `ArrayBuffer`.
- **Python:** `Mipmap.build(bytes, format, size=None)` with `bytes` as `bytes`, `list[int]`, or numpy `ndarray`.
- **Swift / Kotlin:** `Mipmap.build(bytes, format)` or `Mipmap.build(bytes, format, size)`, with `bytes` as `Data` or `[UInt8]` in Swift and a `ByteArray` in Kotlin.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let png = std::fs::read("examples/assets/logo.png")?;
# /*[js]
# const png = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
# */
# /*[python]
# png = open("examples/assets/logo.png", "rb").read()
# */
# /*[swift]
# import Foundation
# let png = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
# */
# /*[kotlin]
# val png = java.io.File("examples/assets/logo.png").readBytes()
# */
use fragmentcolor::{Mipmap, Renderer, TextureFormat};

// Imagine `png` came off your asset loader on a worker thread.

// Decode + mipmap generation. Pure CPU; run it wherever you like.
let chain = Mipmap::build((png.as_slice(), TextureFormat::Rgba8UnormSrgb))?;

// Back on the renderer thread, the upload is just a GPU write.
let renderer = Renderer::new();
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
