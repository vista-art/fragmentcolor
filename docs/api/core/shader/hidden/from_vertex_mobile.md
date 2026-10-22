# Shader::from_vertex_mobile()

Uniffi-friendly constructor that takes an `Arc<Vertex>` and delegates to
`Shader::from_vertex`. Uniffi cannot marshal `&Vertex` (a shared reference)
across the FFI boundary, so the mobile shim accepts the ref-counted form.

Inspects the vertex position dimensionality and optional properties to
generate a minimal WGSL shader. Intended as a fallback and for quick
debugging.

Swift and Kotlin call it as `Shader.fromVertex(vertex)`, the same static
constructor as Rust, JS and Python.

## Example

```rust
// hidden file; no public example
```
