# Texture::size

Return the texture's dimensions as a `Size` (`width`, `height`, and
`depth`, which is absent for a 2D texture and holds the depth of a 3D texture
or the layer count of an array). The size reflects the
allocated GPU resource. It changes when `resize` gives the texture a new
size, and every clone of the handle reads the new size.

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
