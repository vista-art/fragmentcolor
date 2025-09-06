# set_clear_color(color: [f32; 4])

Sets the clear color for this [Pass](https://fragmentcolor.org/docs/api/pass).

When the pass is configured to clear, the render target is cleared to the given RGBA color before drawing.

## Example

```rust
use fragmentcolor::{Renderer, Pass, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;

let shader = Shader::default();
let mut pass = Pass::new("solid background");
pass.add_shader(&shader);

pass.set_clear_color([0.1, 0.2, 0.3, 1.0]);

renderer.render(&pass, &target)?;
# Ok(())
# }
```
