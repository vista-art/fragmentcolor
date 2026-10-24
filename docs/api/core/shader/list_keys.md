# Shader::list_keys() -> [string]

Returns a list of all keys in the [Shader](https://fragmentcolor.org/api/core/shader), including uniform names and struct fields using the dot notation.

`list_keys` never blocks. When another thread holds the shader's storage, it returns an empty list and logs a warning.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Shader;

let shader = Shader::default();
let keys = shader.list_keys();

# assert!(keys.contains(&"resolution".to_string()));
# Ok(())
# }
```
