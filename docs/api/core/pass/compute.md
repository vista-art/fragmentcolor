# Pass::compute(name: &str) -> Pass

Creates a new [Pass](https://fragmentcolor.org/api/core/pass) configured for compute workloads.

Only [Shader](https://fragmentcolor.org/api/core/shader) objects that compile to compute pipelines can be added.

## Example

```rust
use fragmentcolor::{Pass, Shader};

let cs = Shader::new("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap();
let pass = Pass::from_shader("compute", &cs);
```
