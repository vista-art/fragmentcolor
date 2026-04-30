# Shader::fetch(input: string | string[])

## Available for Javascript only

Async constructor for Javascript to build a shader whose parts come from the network.

In most platforms the [Shader](https://fragmentcolor.org/api/core/shader) constructor accepts URLs and registry slugs directly. In WASM the constructor cannot perform network requests because constructors cannot be async, so `Shader.fetch(...)` is the async path.

`fetch` accepts the same input forms as the Rust constructor:

- A single raw shader source string (no fetch happens, but the call is still async-shaped).
- A single URL (`https://...`) or registry slug (`"sdf2d/circle"`).
- An array mixing any of the above.

Each part is fetched (URL or slug) or used directly (raw source). The resolved sources are concatenated in order, deduplicated by hash, and validated as a single WGSL shader.

If validation fails, the error message will indicate the location of the error.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Shader;

// Single URL
let shader = Shader::fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl").await?;

# let _ = shader;
# Ok(())
# }
# fn main() { let _ = pollster::block_on(run()); }
```
