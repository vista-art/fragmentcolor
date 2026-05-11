# Renderer::create_texture

Create a [Texture](https://fragmentcolor.org/api/texture/texture) from various inputs.

- Rust: `create_texture(input)` takes any `impl Into<TextureInput>` — bare bytes / `(bytes, [w, h])` / `(bytes, format)` / `(bytes, format, size)` / paths / URLs / KTX2 / `Mipmap`.
- JS: `await renderer.createTexture(input, options?)` accepts `Uint8Array` bytes, string URL/path, CSS selector, `HTMLImageElement`, `HTMLCanvasElement`, `OffscreenCanvas`, `ImageData`, or a `Mipmap` handle. `options` is an optional `{ size?, format?, mipmaps? }` object — when `size` is set, `input` is treated as raw pixel data.
- Python: `renderer.create_texture(input, size=None, format=None, mipmaps=None)` accepts `bytes`, `str` path, NumPy ndarray shaped `[H, W, C]` where C=1/3/4, or a `Mipmap`. Numpy arrays auto-fill `size`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Renderer;
let image = std::fs::read("./examples/assets/image.png")?;
let renderer = Renderer::new();
let tex = renderer.create_texture(&image).await?;
// use in a shader uniform
// shader.set("tex", &tex)?;
# _ = tex;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
