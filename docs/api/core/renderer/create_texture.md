# Renderer::create_texture

Create a [Texture](https://fragmentcolor.org/api/core/texture) from bytes, a file path, a URL, a KTX2 container, or a pre-built [TextureMipChain](https://fragmentcolor.org/api/core/texture-mip-chain).

When you pass a `size`, the bytes are treated as raw pixels in the chosen format. Without a size, the bytes (or the file/URL contents) are decoded as PNG, JPEG, BMP, HDR, etc.

Decode and mipmap generation run on a background worker on native platforms. On the web they run inline on the caller's thread; move heavy decode into a Web Worker if you need parallelism.

## Per-language shapes

- **Rust:** `create_texture(input)`. `input` accepts bare bytes, `(bytes, [w, h])`, `(bytes, format)`, `(bytes, options)`, a path, a URL, a `TextureMipChain`, or a KTX2 input.
- **JS:** `await renderer.createTexture(input, options?)`. `input` is `Uint8Array` / `ArrayBuffer` / URL string / CSS selector / `HTMLImageElement` / `HTMLCanvasElement` / `OffscreenCanvas` / `ImageData` / a `TextureMipChain` handle. `options` is `{ size?, format?, mipmaps? }`; when `size` is set, `input` is read as raw pixel bytes.
- **Python:** `renderer.create_texture(input, size=None, format=None, mipmaps=None)`. `input` is `bytes` / `list[int]` / `str` (path) / numpy `ndarray[H, W, C]` / `TextureMipChain`. Numpy arrays fill in `size` for you.
- **Swift / Kotlin:** `try await renderer.createTexture(bytes)` or `renderer.createTexture(chain)`. Overloads in the binding wrap the underlying enum, so you write the natural call.

## How the discriminator works

| Form | Treatment |
|------|-----------|
| `bytes` (no size) | Encoded image; decoded internally (PNG, JPEG, BMP, HDR, etc.). |
| `(bytes, size)` / options with `size` set | Raw pixel bytes; `bpp(format) * width * height` long, no decode. |
| `(bytes, format)` / options with `format` set | Encoded image reinterpreted as `format` (e.g. `Rgba8Unorm` for normal-map data, `R16Unorm` for 16-bit grayscale). |
| `path` / `URL` | File or HTTP fetch, then decoded. |
| `TextureMipChain` | Pre-built CPU mip chain; GPU-only upload. Build via [TextureMipChain::prepare](https://fragmentcolor.org/api/core/texture-mip-chain/prepare) on a worker thread. |
| `Ktx2Bytes` / `Ktx2Path` / `Ktx2Url` | KTX2 container (BC / ETC2 / ASTC / uncompressed); the file's declared format and pre-baked mip chain win. |

A full mipmap chain is generated for source images by default (`options.mipmaps = true`). Set to `false` to skip the CPU work for textures that won't be sampled at distance.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Renderer;
let renderer = Renderer::new();
let image = std::fs::read("logo.png")?;
let tex = renderer.create_texture(&image[..]).await?;
# _ = tex.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
