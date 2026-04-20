# AGENTS.md — Shader rules (short)

Ownership & validation
- `Shader` owns `Mesh` attachments; validate at attach time and reject incompatible meshes with a typed `ShaderError`.
- No invalid mesh may be attached to a shader under any circumstance.
- If the shader has no `@location` vertex inputs (fullscreen / builtin-only), attaching a mesh must be rejected.

Constructors
- `Shader::new(source: &str) -> Result<Self, ShaderError>` — WGSL or GLSL source; the core Rust entry point.
- `Shader::from_mesh(mesh: &Mesh) -> Self` — infers a minimal shader from a mesh (returns a default shader on failure and logs the error; never panics).
- `Shader::from_vertex(v: &Vertex) -> Self` — same idea from a single vertex (pos dim + common properties).
- Mobile uniffi: `Shader::new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError>` in `platform/mobile.rs` (owned `String` + `Arc<Self>` as uniffi requires). Swift / Kotlin extensions re-expose it as `Shader(source)`.

Reflection & mapping
- Vertex input mapping order:
  1. instance-by-location
  2. vertex-by-location (position / property)
  3. by-name (instance first, then vertex)
- Formats must match exactly; no implicit conversions.

Errors & API shape
- Return `ShaderError` from shader APIs; avoid `unwrap` / `expect` / `panic`.
- `set()` is non-blocking (queues last-wins updates); reads may transiently return `ShaderError::Busy` under contention — that is expected.
- Keep public methods thin; delegate to internal state.
