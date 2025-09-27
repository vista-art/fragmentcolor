# Pass::is_compute

Returns true if this Pass is a compute pass (has only compute shaders).

## Example

```rust
use fragmentcolor::{Shader, Pass, Mesh, Vertex};

let wgsl = r#"
@compute @workgroup_size(1)
fn cs_main() { }
"#;
let shader = Shader::new(wgsl).unwrap();
let pass = Pass::from_shader("p", &shader);

if pass.is_compute() {
    println!("This is a compute pass.");
}

# assert!(shader.is_compute());
# assert!(pass.is_compute());
```
