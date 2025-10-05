# Pass::set_viewport(viewport: Region)

Sets the viewport region for this [Pass](https://fragmentcolor.org/api/core/pass).

The viewport restricts drawing to a rectangular area of the [Target](https://fragmentcolor.org/api/core/target).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Pass, Shader, Region};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
let mut pass = Pass::new("clipped");
pass.add_shader(&shader);

pass.set_viewport(Region::new((0, 0), (32, 32)));

renderer.render(&pass, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
