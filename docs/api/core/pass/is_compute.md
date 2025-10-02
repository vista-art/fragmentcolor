# Pass::is_compute

Returns true if this Pass is a compute pass (has only compute shaders).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Shader, Pass};

let shader = Shader::new(r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#)?;
let pass = Pass::from_shader("p", &shader);

// Call the method
let is_compute = pass.is_compute();

# _ = is_compute;
# assert!(pass.is_compute());
# Ok(())
# }
```
