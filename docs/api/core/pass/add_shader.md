# Pass::add_shader(shader: Shader)

Add a [Shader](https://fragmentcolor.org/api/core/shader) to the
[Pass](https://fragmentcolor.org/api/core/pass). Shaders run in the order
they were added, sharing the pass's targets, viewport, clear color, and
load policy.

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::new("p");
pass.add_shader(&shader);
```
