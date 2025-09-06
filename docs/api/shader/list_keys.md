# list_keys() -> [string]

Returns a list of all keys in the Shader, including uniform names and struct fields using the dot notation.

## Example

```rust
use fragmentcolor::Shader;

let shader = Shader::default();
let _keys = shader.list_keys();
```
