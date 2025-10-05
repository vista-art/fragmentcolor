# Pass::add_shader(shader: Shader)

Adds a [Shader](https://fragmentcolor.org/api/core/shader) object to the [Pass](https://fragmentcolor.org/api/core/pass).

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::new("p");
pass.add_shader(&shader);
```
