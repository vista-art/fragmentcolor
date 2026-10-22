# Shader::fetch_mobile()

Uniffi async instance method backing `Shader.fetch(input)` on Swift and
Kotlin. Resolves a single URL, slug, file path, or raw
WGSL source and returns a new compiled `Shader`.

uniffi 0.32 can't make a primary constructor async, so this is an async
method on `Arc<Shader>`. Extension shims in `Shader+Extensions.swift` and
`ShaderExtensions.kt` provide the static-factory spelling, so callers never
hold a dummy instance.

## Example

```rust
// hidden file with no public example; see docs/api/core/shader/fetch.md
```
