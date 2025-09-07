# set_viewport(viewport: Region)

Sets the viewport region for this [Pass](https://fragmentcolor.org/api/pass).

The viewport restricts drawing to a rectangular area of the [Target](https://fragmentcolor.org/api/target).

## Example

```rust
use fragmentcolor::{Renderer, Pass, Shader, Region};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;

let shader = Shader::default();
let mut pass = Pass::new("clipped");
pass.add_shader(&shader);

pass.set_viewport(Region::from_region(0, 0, 32, 32));

renderer.render(&pass, &target)?;
# Ok(())
# }
```
