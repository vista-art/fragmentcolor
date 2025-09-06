# Target

The [Target](https://fragmentcolor.org/docs/api/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/docs/api/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/docs/api/texture_target)).

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;

let shader = Shader::default();
renderer.render(&shader, &target)?;
# Ok(())
# }
```
