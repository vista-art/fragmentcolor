# from_shader(name: &str, shader: Shader) -> Pass

Creates a new [Pass](https://fragmentcolor.org/docs/api/pass) from a single [Shader](https://fragmentcolor.org/docs/api/shader).

The created Pass inherits the render/compute type from the provided Shader.

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::from_shader("single", &shader);
```
