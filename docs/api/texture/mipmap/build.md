# Mipmap::build

Build a mip chain on the calling thread, then hand it to [Renderer::create_texture](https://fragmentcolor.org/api/core/renderer#create_texture) for a GPU-only upload. The work is pure CPU (decode if needed, then downsample to 1x1), so call it from wherever you already do heavy work: a worker thread, a thread pool, an async task, a Web Worker. The renderer thread sees only the finished chain.

`build(input)` takes the same input shapes as `Renderer::create_texture` and `Renderer::create_storage_texture`, so you write the call the same way regardless of which entry point you target. Whether `build` decodes the bytes depends on whether you pass a `size`:

- **Encoded path, `size` absent.** The bytes get decoded as PNG / JPEG / BMP / HDR / etc., and the chain inherits the image's dimensions. The format comes from the bytes, as in `create_texture`, so a file name without an extension works. Use this when you have an encoded blob from disk, the network, or a decoder that emits encoded data.
- **Raw path, `size` present.** The bytes are treated as already laid out for `format` at `size`, one layer deep and exactly that long. A file path takes its size from the file, so a path with a size is an `InvalidInput` error. Use this when your decoder already produced pixel-format-matching bytes (typical for tile-cache pipelines that fold mip generation into the same step that produced the tile).

`build` runs synchronously and accepts only sync-friendly inputs: bytes or a file path. URL inputs (which need an async fetch), KTX2 inputs (already pre-baked), an existing chain, an existing texture, and empty inputs all return `TextureError::InvalidInput` with a message pointing at the right entry point.

A decoded image built for `Bgra8Unorm` or `Bgra` stores blue first, as `create_texture` does, and raw bytes keep their order.

Supported formats: `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `R8Unorm`, `Rg8Unorm`, `R16Unorm`, `Rg16Unorm`, `Rgba16Unorm`, `Rgba16Float`, `Rgba32Float`, and the `Rgba`, `Bgra`, `L8` and `Lab` aliases. Anything else returns `TextureError::UnsupportedMipmapFormat`. Decode failures surface as `MalformedImageError`; size and byte-count mismatches as `InvalidInput`.

## Per-language shapes

- **Rust:** `Mipmap::build((bytes, format))` for the encoded path, `Mipmap::build((bytes, format, size))` for the raw path. Tuples implement `Into<TextureInput>` via the `From<T>` impls.
- **JS / Python:** `Mipmap.build(bytes, format, size?)` with `size` optional.
- **Swift / Kotlin:** `Mipmap.build(bytes, format)` for the encoded path, `Mipmap.build(bytes, format, size)` for the raw path. Bytes are `Data` or `[UInt8]` in Swift and a `ByteArray` in Kotlin; `size` is `[w, h]` or a `Size`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let encoded_png_bytes = std::fs::read("examples/assets/logo.png")?;
# let raw_rgba = vec![200; 8 * 8 * 4];
# /*[js]
# const encoded_png_bytes = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
# const raw_rgba = new Uint8Array(8 * 8 * 4).fill(200);
# */
# /*[python]
# encoded_png_bytes = open("examples/assets/logo.png", "rb").read()
# raw_rgba = bytes([200] * (8 * 8 * 4))
# */
# /*[swift]
# import Foundation
# let encoded_png_bytes = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
# let raw_rgba = Data(repeating: 200, count: 8 * 8 * 4)
# */
# /*[kotlin]
# val encoded_png_bytes = java.io.File("examples/assets/logo.png").readBytes()
# val raw_rgba = ByteArray(8 * 8 * 4) { 200.toByte() }
# */
use fragmentcolor::{Mipmap, Renderer, TextureFormat};

// Encoded path: bytes plus the format you want the chain to live in.
// The dimensions come from the decoded image.
let chain = Mipmap::build((encoded_png_bytes.as_slice(), TextureFormat::Rgba8UnormSrgb))?;

// Raw path: include the size so build skips decoding.
let chain_raw = Mipmap::build((raw_rgba.as_slice(), TextureFormat::Rgba8UnormSrgb, [8, 8]))?;

// Either chain uploads the same way.
let renderer = Renderer::new();
let texture = renderer.create_texture(chain).await?;
# _ = texture.size();
# _ = chain_raw.count();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
