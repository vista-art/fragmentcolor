# Texture::aspect

Return the texture's `width / height` as `f32`. Useful for setting up
projection matrices or laying out a sprite without recomputing the ratio
on every frame.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Size};

let renderer = Renderer::new();
// 1x1 RGBA (white) raw pixel bytes
let pixels: &[u8] = &[255,255,255,255];
let tex = renderer.create_texture((pixels, [1, 1])).await?;
let a = tex.aspect();

# assert!(a > 0.0);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
