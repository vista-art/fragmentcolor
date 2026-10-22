# Texture::size

Return the texture's dimensions as a `Size` (`width`, `height`, and
`depth`, which is 1 for a 2D texture and the layer count for 3D or array
textures). The size reflects the
allocated GPU resource and is constant for the lifetime of the texture.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Size};
let renderer = Renderer::new();
let pixels: &[u8] = &[255,255,255,255];
let tex = renderer.create_texture((pixels, [1, 1])).await?;
let sz = tex.size();
let width = sz.width;
let height = sz.height;
# assert_eq!([width, height], [1, 1]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
