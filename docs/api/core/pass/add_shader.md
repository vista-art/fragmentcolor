# Pass::add_shader(shader: Shader)

Add a [Shader](https://fragmentcolor.org/api/core/shader) to the
[Pass](https://fragmentcolor.org/api/core/pass). Shaders run in the order
they were added, sharing the pass's targets, viewport, clear color, and
load policy.

A shader that declares the light roots, as
[`Material::custom`](https://fragmentcolor.org/api/scene/material#custom)
lists them, takes every [Light](https://fragmentcolor.org/api/scene/light)
the pass holds. When those lights would take it past the most it holds on
the device, the pass logs a warning and leaves the shader out.

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::new("p");
pass.add_shader(&shader);
```
