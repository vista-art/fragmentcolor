# new(source: string)

Creates a new [Shader](https://fragmentcolor.org/api/core/shader) instance from the given WGSL source string, file path, or URL.

GLSL is also supported if you enable the `glsl` feature.
Shadertoy-flavored GLSL is supported if the `shadertoy` feature is enabled.

If the optional features are enabled, the constructor will try to automatically
detect the shader type and parse it accordingly.

If an exception occurs during parsing, the error message will indicate the location of the error.

If the initial source validation passes, the shader is guaranteed to work on the GPU. All uniforms are initialized to their default zero values.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::Shader;

let _shader = Shader::default();
# Ok(())
# }
```

## Platform-specific: Web

In WASM, the constructor cannot fetch a URL directly. Use [Shader](https://fragmentcolor.org/api/core/shader)::[fetch](https://fragmentcolor.org/api/shader/fetch) instead.
