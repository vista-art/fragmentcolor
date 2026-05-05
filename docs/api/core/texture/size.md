# Texture::size

Return the texture's dimensions as a `Size` (`width`, `height`, and an
optional `depth` for 3D or array textures). The size reflects the
allocated GPU resource and is constant for the lifetime of the texture.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Size};
let renderer = Renderer::new();
let pixels: &[u8] = &[255,255,255,255];
let tex = renderer.create_texture((pixels, [1, 1])).await?;
let sz = tex.size();
# assert_eq!([sz.width, sz.height], [1, 1]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
