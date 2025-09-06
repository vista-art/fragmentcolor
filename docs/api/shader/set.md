# set(key: string, value: any)

Sets the value of the uniform identified by the given key.

If the key does not exist or the value format is incorrect, the `set` method throws an exception. The shader remains valid, and if the exception is caught, the shader can still be used with the renderer.

## Example

```rust
use fragmentcolor::Shader;

let shader = Shader::default();
let _ = shader.set("resolution", [800.0, 600.0]);
```
