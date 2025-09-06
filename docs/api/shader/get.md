# get(key: string) -> any

Returns the current value of the uniform identified by the given key.

## Example

```rust
use fragmentcolor::Shader;

let shader = Shader::default();
let _ = shader.set("resolution", [800.0, 600.0]);
let _res: Result<[f32; 2], _> = shader.get("resolution");
```
