# Shader::fetch_compose_mobile()

Uniffi async instance method backing `Shader.fetch([parts])` on Swift and
`ShaderFetch(listOf(...))` on Kotlin. Resolves each element of `parts`
independently (URL, slug, path, or raw source) and returns a new compiled
`Shader` from the merged WGSL.

## Example

```rust
// hidden file; no public example — see docs/api/core/shader/fetch.md
```
