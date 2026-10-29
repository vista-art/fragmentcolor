# Shader::get(key: string) -> any

Returns the current value of the uniform identified by the given key.

A struct or array root reads back as its bytes, and `set` doesn't take
those bytes back for that root. Set a struct one field at a time
(`"light.color"`) and an array one element at a time (`"weights[0]"`).
Only a storage-buffer or push-constant root takes its whole bytes.

A texture slot reads back the id of what it binds. A `Material` map's slot
holds the map's id from the setter call on, before any `Renderer` loads it.

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
