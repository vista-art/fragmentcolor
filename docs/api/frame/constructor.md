# Frame::new()

Creates a new [Frame](https://fragmentcolor.org/api/frame) object.

A [Frame](https://fragmentcolor.org/api/frame) is an ordered collection of [Pass](https://fragmentcolor.org/api/pass) objects that will be rendered by the [Renderer](https://fragmentcolor.org/api/renderer) in sequence.

## Example

```rust
use fragmentcolor::{Renderer, Frame, Pass, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([10, 10]))?;

let shader = Shader::default();
let mut pass = Pass::new("single pass");
pass.add_shader(&shader);

let mut frame = Frame::new();
frame.add_pass(&pass);

renderer.render(&frame, &target)?;
# Ok(())
# }
```
