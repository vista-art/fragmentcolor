# Renderer::create_texture_with_format

Create a [Texture](https://fragmentcolor.org/api/core/texture) from various inputs.

- Rust: `create_texture(input)` infers from encoded bytes or file path; use `create_texture_with(input, Some(size), Some(format))` for raw pixel bytes.
- JS: `await renderer.createTexture(input)` accepts `Uint8Array` bytes, string URL/path, or a CSS selector/HTMLImageElement
- Python: `renderer.create_texture(input)` accepts `bytes`, `str` path, or a NumPy ndarray shaped `[H, W, C]` where C=1/3/4.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Size, TextureFormat};
let renderer = Renderer::new();
let image = std::fs::read("logo.png")?;
let tex = renderer.create_texture_with_format(&image, TextureFormat::Rgba).await?;
# _ = tex.size();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
