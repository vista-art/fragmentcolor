# add_pass(pass: Pass)

Adds a [Pass](https://fragmentcolor.org/api/pass) to this [Frame](https://fragmentcolor.org/api/frame).

Passes are rendered in the order they are added.

## Example

```rust
use fragmentcolor::{Renderer, Frame, Pass, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([10, 10]).await?;

let shader = Shader::default();
let mut pass1 = Pass::new("first");
pass1.add_shader(&shader);

let mut pass2 = Pass::new("second");
pass2.add_shader(&shader);

let mut frame = Frame::new();
frame.add_pass(&pass1);
frame.add_pass(&pass2);

renderer.render(&frame, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
