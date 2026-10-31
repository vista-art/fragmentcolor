# Shader::from_mesh_mobile()

Uniffi-friendly constructor that takes an `Arc<Mesh>` and delegates to
`Shader::from_mesh`. Uniffi cannot marshal `&Mesh` (a shared reference)
across the FFI boundary, so the mobile shim accepts the ref-counted form.

Automatically attaches the mesh to the new shader; callers do not need a
separate `addMesh` call. An empty mesh, or one the new shader refuses, throws
`FragmentColorError` in Swift and `FragmentColorException` in Kotlin.

Swift and Kotlin call it as `Shader.fromMesh(mesh)`, the same static
constructor as Rust, JS and Python.

## Example

```rust
// hidden file; no public example
```
