# Shader::fetch(input)

Async constructor that returns a compiled `Shader` from one or more parts.
Each part can be:

- a raw WGSL source string
- a registry slug like `"sdf2d/circle"`
- an `https://` URL pointing at a `.wgsl` file
- a local file path ending in `.wgsl`, `.glsl`, `.frag`, or `.vert`

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
| Kotlin   | `Shader.fetch(input)` | `suspend fun` via uniffi async method |

> **Note (Swift / Kotlin):** the underlying uniffi binding is an `async`
> instance method. The Swift and Kotlin `Shader.fetch(...)` extensions call
> it on a throwaway default shader, so callers see a static factory.

## Where each input resolves

| Platform | Registry slugs and registry URLs | Other `https://` URLs | File paths |
|----------|----------------------------------|-----------------------|------------|
| Rust (desktop) | Bundled shader library, offline | Fetched with the `network` feature, an error without it | Read from disk |
| Python | Bundled shader library, offline | Fetched by a wheel built with the `network` feature, an error otherwise | Read from disk |
| Swift / Kotlin | Bundled shader library, offline | Not fetched; a network error | Read from disk |
| Web, `Shader.fetch` | The browser's `fetch()` | The browser's `fetch()` | Fetched relative to the page's origin |
| Web, `new Shader` | Not fetched; an error that points at `Shader.fetch` | Not fetched; an error | Not fetched; an error |

After `Shader::set_registry`, slugs and registry URLs are fetched first on
every platform, and the bundled copy stands in when the fetch fails. A slug
resolves against the new registry. A registry URL is fetched as written, so
`https://fragmentcolor.org/shaders/sdf2d/circle.wgsl` still asks the default registry.

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
[Shader::new](https://fragmentcolor.org/api/core/shader#shadernewinput-string--string).
