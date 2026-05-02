# Renderer::create_texture

Create a [Texture](https://fragmentcolor.org/api/core/texture) from any input shape — there is exactly one method.

The single `create_texture(spec)` entry point covers every input shape. The spec's discriminator: the presence of `size` selects raw-pixel-bytes interpretation; otherwise the input is treated as encoded image data (PNG / JPEG / etc.), a file path, a URL, a KTX2 container, or a pre-built [TextureMipChain](https://fragmentcolor.org/api/core/texture-mip-chain).

The CPU work (decode, mipmap chain, raw-byte wrap, KTX2 parse) runs on a background worker on every native target — the calling thread is never pinned on `queue.write_texture`. On the web, prep runs inline (wgpu types are `!Send` on `wasm32`); move heavier work into a Web Worker yourself if you need real parallelism.

## Per-language shapes

- **Rust:** `create_texture(impl Into<TextureInput>)`. The `From<T>` impls give you `(bytes)`, `(bytes, [w, h])`, `(bytes, format)`, `(bytes, options)`, plus paths, URLs, `TextureMipChain`, KTX2 inputs, and explicit `TextureInput { input, options }` literals.
- **JS:** `await renderer.createTexture(input, options?)`. `input` is `Uint8Array` / `ArrayBuffer` / URL string / CSS selector / `HTMLImageElement` / `HTMLCanvasElement` / `OffscreenCanvas` / `ImageData` / a `TextureMipChain` handle. `options` is an object `{ size?, format?, mipmaps? }`. When `size` is set, `input` is treated as raw pixel bytes.
- **Python:** `renderer.create_texture(input, size=None, format=None, mipmaps=None)`. `input` is `bytes` / `list[int]` / `str` (path) / numpy `ndarray[H, W, C]` / `TextureMipChain`. Numpy arrays auto-fill `size` if you don't pass it.
- **Swift / Kotlin (uniffi):** `try await renderer.createTexture(input:options:)` where `input` is `TextureInputMobile` (an enum mirroring `TextureInput` minus the Rust-only `DynamicImage` variant). Swift/Kotlin extension files supply natural overloads (`renderer.createTexture(bytes)`, `renderer.createTexture(chain)`) so callers never see the enum directly.

## How the discriminator works

| Form | Treatment |
|------|-----------|
| `bytes` (no size) | Encoded image — decoded internally (PNG, JPEG, BMP, HDR, etc.). |
| `(bytes, size)` / options with `size` set | Raw pixel bytes — `bpp(format) * width * height` long, no decode. |
| `(bytes, format)` / options with `format` set | Encoded image, but interpret as `format` (e.g. `Rgba8Unorm` for normal-map data, `R16Unorm` for 16-bit grayscale). |
| `path` / `URL` | File or HTTP fetch, then decoded. |
| `TextureMipChain` | Pre-built CPU mip chain — GPU-only upload. Build via [TextureMipChain::prepare](https://fragmentcolor.org/api/core/texture-mip-chain/prepare) on a worker thread. |
| `Ktx2Bytes` / `Ktx2Path` / `Ktx2Url` | KTX2 container (BC / ETC2 / ASTC / uncompressed); the file's declared format and pre-baked mip chain win. |

A full mipmap chain is generated for source images by default (`options.mipmaps = true`). Set to `false` to skip the CPU work for textures that won't be sampled at distance.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Renderer;
let renderer = Renderer::new();
// Encoded image bytes (PNG / JPEG / etc.) — single tuple, no extra method.
let image = std::fs::read("logo.png")?;
let tex = renderer.create_texture(&image[..]).await?;
# _ = tex.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
