# Shader::fetch_mobile()

Uniffi async instance method backing `Shader.fetch(input:)` on Swift and
`ShaderFetch(input)` on Kotlin. Resolves a single URL, slug, file path, or raw
WGSL source and returns a new compiled `Shader`.

Uniffi 0.31 does not support async constructors, so this is expressed as an
async method on `Arc<Shader>`. Extension shims in `Shader+Extensions.swift`
and `ShaderExtensions.kt` provide clean static-factory spellings so callers
never need to hold a dummy instance.

## Example

```rust
// hidden file; no public example — see docs/api/core/shader/fetch.md
```
