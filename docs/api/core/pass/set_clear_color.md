# Pass::set_clear_color(color: [f32; 4])

Sets the clear color for this [Pass](https://fragmentcolor.org/api/core/pass).

When the pass is configured to clear, the render target is cleared to the given RGBA color before drawing.

The color is linear, as `[r, g, b, a]` or as `[r, g, b]` with an opaque alpha. Each channel reaches the GPU as given. A float target such as `Rgba16Float` keeps a value above 1 or below 0, and a unorm target such as `Rgba8Unorm` clamps it to the 0 to 1 range when it clears.

A channel that is NaN or infinite is an error that leaves the pass unchanged.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Pass, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
let mut pass = Pass::new("solid background");
pass.add_shader(&shader);

pass.set_clear_color([0.1, 0.2, 0.3, 1.0])?;

renderer.render(&pass, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
