# Shader::is_compute

Returns true if this Shader is a compute shader (has a compute entry point).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let shader = Shader::new(r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#)?;

// Call the method
let is_compute = shader.is_compute();

# let _ = is_compute;
# assert!(shader.is_compute());
# Ok(())
# }
```
