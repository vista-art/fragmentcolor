# Frame::new()

Creates a new [Frame](https://fragmentcolor.org/api/core/frame) object.

A [Frame](https://fragmentcolor.org/api/core/frame) is an ordered collection of [Pass](https://fragmentcolor.org/api/core/pass) objects that will be rendered by the [Renderer](https://fragmentcolor.org/api/core/renderer) in sequence.

## Example

```rust
use fragmentcolor::{Renderer, Frame, Pass, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([10, 10]).await?;

let shader = Shader::default();
let mut pass = Pass::new("single pass");
pass.add_shader(&shader);

let mut frame = Frame::new();
frame.add_pass(&pass);

renderer.render(&frame, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
