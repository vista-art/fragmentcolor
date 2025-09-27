# Shader::is_compute

Returns true if this Shader is a compute shader (has a compute entry point).

## Example

```rust
use fragmentcolor::{Shader, Pass, Mesh, Vertex};

let wgsl = r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#;
let shader = Shader::new(wgsl).unwrap();

if shader.is_compute() {
    println!("This is a compute shader.");
}

# let pass = Pass::from_shader("p", &shader);
# assert!(shader.is_compute());
# assert!(pass.is_compute());
```
