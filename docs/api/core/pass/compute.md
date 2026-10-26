# Pass::compute(name: &str) -> Pass

Creates a new [Pass](https://fragmentcolor.org/api/core/pass) configured for compute workloads.

Only [Shader](https://fragmentcolor.org/api/core/shader) objects that compile to compute pipelines can be added.

The first shader added decides the pass kind, whichever constructor made the
pass. After that, `add_shader` returns `PassError::ShaderKind` for a shader of
the other kind and leaves the pass as it was.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Pass, Shader};

let cs = Shader::new("@compute @workgroup_size(8,8,1) fn cs_main() {}")?;
let pass = Pass::compute("compute");
pass.add_shader(&cs)?;
# assert!(pass.is_compute());
# Ok(())
# }
```
