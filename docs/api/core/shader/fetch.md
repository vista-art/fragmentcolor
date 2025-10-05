# Shader::fetch(url: string)

## Available for Javascript only

This function is an alternative constructor for Javascript to fetch a shader from a URL.

In most platforms, the constructor accepts a URL directly. In WASM, however, it's not possible to perform network requests in a constructor because constructors cannot be async or create blocking async closures internally.

This function creates a new [Shader](https://fragmentcolor.org/api/core/shader) instance from the given URL.

If an exception occurs during parsing, the error message will indicate the location of the error.

If the initial source validation passes, the shader is guaranteed to work on the GPU. All uniforms are initialized to their default zero values.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader};

let shader = Shader::fetch("https://fragmentcolor.org/shaders/circle.wgsl").await?;

# Ok(())
# }
# fn main() -> { Ok(pollster::block_on(run())) }
```
