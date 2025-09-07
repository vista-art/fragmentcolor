# load_previous()

Configures this [Pass](https://fragmentcolor.org/api/pass) to load the previous contents of the [Target](https://fragmentcolor.org/api/target) instead of clearing it.

This is useful when layering multiple passes where the next pass should blend with the prior results.

## Example

```rust
use fragmentcolor::{Renderer, Pass, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;

let shader = Shader::default();
let mut pass = Pass::new("blend with previous");
pass.add_shader(&shader);
pass.load_previous();

renderer.render(&pass, &target)?;
# Ok(())
# }
```
