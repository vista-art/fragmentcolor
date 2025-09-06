# add_shader(shader: Shader)

Adds a Shader object to the Pass.

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::new("p");
pass.add_shader(&shader);
```
