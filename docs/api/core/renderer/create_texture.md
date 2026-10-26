# Renderer::create_texture

Create a [Texture](https://fragmentcolor.org/api/texture/texture) from bytes, a file path, a URL, a KTX2 container, or a pre-built [Mipmap](https://fragmentcolor.org/api/texture/mipmap).

When you pass a `size`, the bytes are treated as raw pixels in the chosen format. Without a size, the bytes (or the file/URL contents) are decoded as PNG, JPEG, BMP, HDR, etc. A size applies to bytes only: a path, a URL, a KTX2 file and a `Mipmap` carry their own, so a size with one is an `InvalidInput` error, raised before any file read or fetch.

A size with a side of zero, or one past the device's limits, is an error that reaches no GPU call, and so is an image larger than the device takes, which is checked from its header before the pixels decode: `TextureError::InvalidInput` inside `RendererError::TextureError` in Rust, a thrown `Error` in JavaScript, `InvalidInputError` (a `FragmentColorError`) in Python, `FragmentColorError` in Swift and `FragmentColorException` in Kotlin.

Decode and mipmap generation run on a background worker on native platforms. On the web they run inline on the caller's thread; move heavy decode into a Web Worker if you need parallelism.

## Per-language shapes

- **Rust:** `create_texture(input)`. `input` accepts bare bytes, `(bytes, [w, h])`, `(bytes, format)`, `(bytes, options)`, a path, a URL string, a `Texture`, a `Mipmap`, or a KTX2 input. A string that starts with `http://` or `https://` is a URL; any other string is a path.
- **JS:** `await renderer.createTexture(input, options?)`. `input` is `Uint8Array` / `ArrayBuffer` / URL string / CSS selector / `HTMLImageElement` / `HTMLCanvasElement` / `OffscreenCanvas` / `ImageData` / a `Texture` / a `Mipmap`. `options` is `{ size?, format?, mipmaps?, sampler?, usage? }`; when `size` is set, `input` is read as raw pixel bytes. `ImageData` and canvases bring their own size and RGBA pixels, so their options may repeat that size and pick an 8-bit RGBA or BGRA format, and anything else throws. The options object takes the size under `size`, and a `width` or `height` beside another option throws.
- **Python:** `renderer.create_texture(input, size=None, format=None, mipmaps=None, sampler=None, usage=None)`. `input` is `bytes` / `bytearray` / `memoryview` / a list or tuple of ints / a path or URL `str` / `os.PathLike` / uint8 numpy `ndarray[H, W, C]` / `Texture` / `Mipmap`. Numpy arrays fill in `size` for you, and a `size` that differs from the array's raises `InvalidInputError`. An array whose channels match the format's bytes per pixel uploads as it is, such as an `(H, W)` array as `R8Unorm`, and one with 1 or 3 channels gains an opaque alpha for an 8-bit RGBA or BGRA format. Any other shape raises `InvalidInputError`.
- **Swift / Kotlin:** `try await renderer.createTexture(bytes)`, `createTexture(bytes, format)`, `createTexture(bytes, [w, h])`, `createTexture(bytes, options)`, `createTexture(path)` or `createTexture(chain)`. Bytes are `Data` or `[UInt8]` in Swift and `ByteArray` in Kotlin. Every bytes form also takes an optional `usage`.

## How the discriminator works

| Form | Treatment |
|------|-----------|
| `bytes` (no size) | Encoded image; decoded internally (PNG, JPEG, BMP, HDR, etc.). |
| `(bytes, size)` / options with `size` set | Raw pixel bytes; exactly `bpp(format) * width * height * depth` long, no decode, and any other length is an `InvalidInput` error. The default `Rgba` format uploads them as linear `Rgba8Unorm`. |
| `(bytes, format)` / options with `format` set | Encoded image reinterpreted as `format` (e.g. `Rgba8Unorm` for normal-map data, `R16Unorm` for 16-bit grayscale, `Rgba32Float` for full float precision). |
| `path` / `URL` | File or HTTP fetch, then decoded. A size with it is an error. |
| `Mipmap` | Pre-built CPU mip chain; GPU-only upload. Build via [Mipmap::build](https://fragmentcolor.org/api/texture/mipmap#build) on a worker thread. |
| `Ktx2Bytes` / `Ktx2Path` / `Ktx2Url` | KTX2 container (BC / ETC2 / ASTC / uncompressed); the file's declared format and pre-baked mip chain win. |

A full mipmap chain is generated for source images by default (`options.mipmaps = true`). Set to `false` to skip the CPU work for textures that won't be sampled at distance.

An HDR image decodes into `Rgba16Float`. A value past the half float range saturates at 65504, or -65504, and never becomes infinity, the way Three.js converts it. An explicit `Rgba32Float` keeps the full range, and a linear sampler filters it where the device has `FLOAT32_FILTERABLE`.

A decoded image, an `ImageData` or a canvas uploads into `Bgra8Unorm` blue first, so its colors sample right, while raw bytes upload as given. A decoded image fills the 8-bit and 16-bit color formats and `Rgba16Float` or `Rgba32Float`. An integer format such as `Rgba32Uint`, or `Depth32Float`, is an `InvalidInput` error: pass raw bytes with a size for an integer texture, and draw into a depth texture.

`options.usage` adds `TextureUsage` flags to the `TEXTURE_BINDING | COPY_DST` every texture made from data has, such as `COPY_SRC` to read it back. A bit outside the five `TextureUsage` flags is an `InvalidInput` error.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let image = std::fs::read("examples/assets/logo.png")?;
# /*[js]
# const image = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
# */
# /*[python]
# image = open("examples/assets/logo.png", "rb").read()
# */
# /*[swift]
# import Foundation
# let image = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
# */
# /*[kotlin]
# val image = java.io.File("examples/assets/logo.png").readBytes()
# */
use fragmentcolor::Renderer;
let renderer = Renderer::new();
let tex = renderer.create_texture(&image).await?;
# _ = tex.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
