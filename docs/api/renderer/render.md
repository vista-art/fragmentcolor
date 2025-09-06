# render(renderable: Shader | Pass | Frame, target: Target)

Renders the given object to the given [Target](https://fragmentcolor.org/docs/api/target).

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([10, 10]))?;
let shader = Shader::default();
renderer.render(&shader, &target)?;
# Ok(())
# }
```
