# render(renderable: Shader | Pass | Frame, target: Target)

Renders the given object to the given [Target](https://fragmentcolor.org/api/core/target).

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([10, 10]).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
