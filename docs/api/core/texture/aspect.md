# Texture::aspect

Returns width/height as f32.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Size};

let renderer = Renderer::new();
// 1x1 RGBA (white) raw pixel bytes
let pixels: &[u8] = &[255,255,255,255];
let tex = renderer.create_texture_with_size(pixels, [1, 1]).await?;
let a = tex.aspect();

# assert!(a > 0.0);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
