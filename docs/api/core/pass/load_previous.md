# Pass::load_previous()

Configures this [Pass](https://fragmentcolor.org/api/core/pass) to load the previous contents of the [Target](https://fragmentcolor.org/api/core/target) instead of clearing it.

This is useful when layering multiple passes where the next pass should blend with the prior results.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Pass, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
let mut pass = Pass::new("blend with previous");
pass.add_shader(&shader);
pass.load_previous();

renderer.render(&pass, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
