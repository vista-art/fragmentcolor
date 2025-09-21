# Pass::set_compute_dispatch

Set the compute dispatch size for a compute pass.

## Example

```rust
use fragmentcolor::{Pass, Shader};
let cs = Shader::new("@compute @workgroup_size(8,8,1) fn cs_main() {}").unwrap();
let pass = Pass::from_shader("compute", &cs);
pass.set_compute_dispatch(64, 64, 1);
```
