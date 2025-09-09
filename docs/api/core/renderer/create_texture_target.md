# Headless rendering: create_texture_target

Render to an offscreen texture without a Window or Canvas.

This is useful for tests, server-side rendering, or running examples in CI.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader};
let renderer = Renderer::new();

// Create an offscreen texture target with a size of 64x64 pixels.
let target = renderer.create_texture_target([64, 64]).await?;

renderer.render(&Shader::default(), &target)?;
let image = target.get_image(); // get the rendered image

# assert_eq!(image.len(), 64 * 64 * 4); // RGBA8
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
