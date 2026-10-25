# Renderer::create_texture_target(size: [u32; 2])

Render to an offscreen texture without a Window or Canvas.

This is useful for tests, server-side rendering, or running examples in CI.

A width or height of zero, or a side past the device limit, returns `TextureError::InvalidInput`, the error `resize` returns.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Target};
let renderer = Renderer::new();

// Create an offscreen texture target with a size of 64x64 pixels.
let target = renderer.create_texture_target([64, 64]).await?;

renderer.render(&Shader::default(), &target)?;

// get the rendered image
let image = target.get_image().await?;

# // RGBA8
# assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
