# Headless rendering: create_texture_target

Render to an offscreen texture without a Window or Canvas.

This is useful for tests, server-side rendering, or running examples in CI.

## Rust

```rust path=null start=null
use fragmentcolor::{Renderer, Shader, Pass, Frame};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;

let shader = Shader::default();
renderer.render(&shader, &target)?;

let pass = Pass::new("single pass");
pass.add_shader(&shader);
renderer.render(&pass, &target)?;

let mut frame = Frame::new();
frame.add_pass(&pass);
renderer.render(&frame, &target)?;
# Ok(())
# }
```

## Python

```python path=null start=null
from fragmentcolor import Renderer, Shader, Pass, Frame

renderer = Renderer()
# size can be a tuple or list
target = renderer.create_texture_target((64, 64))

shader = Shader("circle.wgsl")
shader.set("resolution", [64.0, 64.0])
shader.set("circle.radius", 10.0)
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8])
shader.set("circle.border", 2.0)
shader.set("circle.position", [0.0, 0.0])

renderer.render(shader, target)

rpass = Pass("single pass")
rpass.add_shader(shader)
renderer.render(rpass, target)

frame = Frame()
frame.add_pass(rpass)
renderer.render(frame, target)
```

## Javascript (Web)

```js path=null start=null
import init, { Renderer, Shader, Pass, Frame } from "fragmentcolor";

await init();

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64, 64]);

const shader = new Shader("circle.wgsl");
shader.set("resolution", [64.0, 64.0]);
shader.set("circle.radius", 10.0);
shader.set("circle.color", [1.0, 0.0, 0.0, 0.8]);
shader.set("circle.border", 2.0);
shader.set("circle.position", [0.0, 0.0]);

renderer.render(shader, target);

const rpass = new Pass("single pass");
rpass.add_shader(shader);
renderer.render(rpass, target);

const frame = new Frame();
frame.add_pass(rpass);
renderer.render(frame, target);
```
