# Shader::fetch(input)

Async constructor that resolves a URL, registry slug, file path, or raw WGSL
source and returns a compiled `Shader`. Available on all four platforms.

Each part of `input` is classified as one of:

- **Raw WGSL source** — used directly, no I/O.
- **URL** (`https://...`) — fetched over the network.
- **Registry slug** (`"category/name"`) — looked up in the slug registry (or
  served from the embedded library if the matching `shaders-<category>` feature
  is compiled in).
- **File path** (starts with `/`, `./`, `../`, `~/`, or ends with `.wgsl` /
  `.glsl` / `.frag` / `.vert`) — read from disk (native platforms only).

Parts are deduplicated by source hash and concatenated in order before
compilation. On WASM, `Shader::new` cannot perform network I/O because
constructors cannot be async; `Shader.fetch` is the required path there.
On Python, Swift, and Kotlin, `Shader.fetch` and `Shader.new` are both
available: `new` resolves URLs synchronously (blocking), while `fetch`
is the idiomatic async path.

## Platform notes

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

// Single URL
let shader = Shader::fetch("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl").await?;

// Registry slug
let shader2 = Shader::fetch("sdf2d/circle").await?;

# let _ = (shader, shader2);
# Ok(())
# }
# fn main() { let _ = pollster::block_on(run()); }
```
