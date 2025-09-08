# Headless rendering: create_texture_target

Render to an offscreen texture without a Window or Canvas.

This is useful for tests, server-side rendering, or running examples in CI.

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;
let _image = target.get_image();
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
