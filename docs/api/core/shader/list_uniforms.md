# Shader::list_uniforms() -> [string]

Returns a list of all uniform names in the [Shader](https://fragmentcolor.org/api/core/shader) (excluding struct fields).

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let shader = Shader::default();
let list = shader.list_uniforms();

# assert!(list.contains(&"resolution".to_string()));
# Ok(())
# }
```
