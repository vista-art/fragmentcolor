# Pass

The [Pass](https://fragmentcolor.org/api/core/pass) object is a collection of [Shader](https://fragmentcolor.org/api/core/shader) objects that are rendered to a [Target](https://fragmentcolor.org/api/core/target) by the [Renderer](https://fragmentcolor.org/api/core/renderer).

While the [Shader](https://fragmentcolor.org/api/core/shader) represents a single **Render Pipeline** or a **Compute Pipeline**,
the [Pass](https://fragmentcolor.org/api/core/pass) can be used to draw multiple Shaders in sequence,
for example when you have multiple objects in a scene with different materials.

The [Pass](https://fragmentcolor.org/api/core/pass) represents a single RenderPass or a ComputePass in the WebGPU API.

The constructor creates a RenderPass by default. To create a ComputePass, call [Pass](https://fragmentcolor.org/api/core/pass)::compute().

After creation, it will only accept a compatible [Shader](https://fragmentcolor.org/api/core/shader) object. If you try to add a Compute [Shader](https://fragmentcolor.org/api/core/shader) to a Render [Pass](https://fragmentcolor.org/api/core/pass) or vice-versa,
it won't add the shader to its internal list and log a warning message in the console.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{ Shader, Pass, Renderer, Frame };

let renderer = Renderer::new();
let window = fragmentcolor::headless_window([100, 100]);
let target = renderer.create_target(window).await?;
let shader = Shader::default();

let mut pass = Pass::new("First Pass");
pass.add_shader(&shader);

let mut pass2 = Pass::new("Second Pass");
pass2.add_shader(&shader);

// standalone
renderer.render(&pass, &target)?;

// using a Frame
let mut frame = Frame::new();
frame.add_pass(&pass);
frame.add_pass(&pass2);
renderer.render(&frame, &target)?;

// vector of passes (consume them)
renderer.render(&vec![pass, pass2], &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
