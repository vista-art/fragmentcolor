# Shader::fetch(input)

Async constructor that returns a compiled `Shader` from one or more parts.
Each part can be:

- a raw WGSL source string
- a registry slug like `"sdf2d/circle"`
- an `https://` URL pointing at a `.wgsl` file
- a local file path ending in `.wgsl`, `.glsl`, `.frag`, or `.vert`
  (native platforms only)

Parts are deduplicated by source hash and concatenated in order before
compilation. `fetch` is the async path used when any part needs network or
file I/O. `Shader::new` covers the same shapes for callers that prefer a
synchronous constructor.

## Platforms

| Platform | Spelling | Async mechanism |
|----------|----------|-----------------|
| Web (JS) | `await Shader.fetch(input)` | `Promise` via `wasm_bindgen` async |
| Python   | `Shader.fetch(input)` | Blocks the calling thread via `pollster::block_on` |
| Swift    | `try await Shader.fetch(input)` | `async throws` via uniffi async method |
| Kotlin   | `ShaderFetch(input)` | `suspend fun` via uniffi async method |

> **Note (Swift / Kotlin):** uniffi 0.31 does not support async constructors.
> The underlying uniffi binding is an `async` instance method; the Swift
> `Shader.fetch(_:)` and Kotlin `ShaderFetch(...)` wrappers handle the
> throw-away receiver internally so callers see a clean static factory.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Shader;

// Full registry URL.
let shader = Shader::fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl").await?;

// Equivalent shorthand using the registry slug.
let shader2 = Shader::fetch("sdf2d/circle").await?;

# let _ = (shader, shader2);
# Ok(())
# }
# fn main() { let _ = pollster::block_on(run()); }
```

For composition (passing multiple parts as an array), see
[Shader::new](https://fragmentcolor.org/api/core/shader#shadernew).
