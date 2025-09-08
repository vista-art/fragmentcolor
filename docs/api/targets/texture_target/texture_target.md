# TextureTarget

The [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) is an offscreen [Target](https://fragmentcolor.org/api/core/target) backed by a GPU texture.

Use it for headless rendering, tests, server-side image generation, or CI.

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
renderer.render(&shader, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
