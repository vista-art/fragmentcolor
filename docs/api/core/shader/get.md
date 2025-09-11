# Shader::get(key: string) -> any

Returns the current value of the uniform identified by the given key.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let shader = Shader::default();
shader.set("resolution", [800.0, 600.0])?;
let res: [f32; 2] = shader.get("resolution")?;

# assert_eq!(res, [800.0, 600.0]);
# Ok(())
# }
```
