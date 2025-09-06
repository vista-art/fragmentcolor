# list_uniforms() -> [string]

Returns a list of all uniform names in the Shader (excluding struct fields).

## Example

```rust
use fragmentcolor::Shader;

let shader = Shader::default();
let _list = shader.list_uniforms();
```
