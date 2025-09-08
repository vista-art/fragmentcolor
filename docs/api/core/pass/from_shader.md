# from_shader(name: &str, shader: Shader) -> Pass

Creates a new [Pass](https://fragmentcolor.org/api/pass) from a single [Shader](https://fragmentcolor.org/api/shader).

The created [Pass](https://fragmentcolor.org/api/pass) inherits the render/compute type from the provided [Shader](https://fragmentcolor.org/api/shader).

## Example

```rust
use fragmentcolor::{Pass, Shader};

let shader = Shader::default();
let pass = Pass::from_shader("single", &shader);
```
