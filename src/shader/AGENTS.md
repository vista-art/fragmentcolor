# AGENTS.md — Shader rules (short)

Ownership & validation
- `Shader` owns `Mesh` attachments; validate at attach time and reject incompatible meshes with a typed `ShaderError`.
- No invalid mesh may be attached to a shader under any circumstance.
- If the shader has no `@location` vertex inputs (fullscreen / builtin-only), attaching a mesh must be rejected.
- Attach checks every vertex and instance value, not only the first, and each one must hold every attribute the shader reads. A failed attach records nothing on the mesh, so `Pass::add_mesh` can try the next shader.

Constructors
- `Shader::new(source: &str) -> Result<Self, ShaderError>` — WGSL or GLSL source; the core Rust entry point.
- `Shader::from_mesh(mesh: &Mesh) -> Self` — infers a minimal shader from a mesh (returns a default shader on failure and logs the error; never panics).
- `Shader::from_vertex(v: &Vertex) -> Self` — same idea from a single vertex (pos dim + common properties).
- Mobile uniffi: `Shader::new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError>` in `platform/mobile.rs` (owned `String` + `Arc<Self>` as uniffi requires). Swift / Kotlin extensions re-expose it as `Shader(source)`.

Reflection & mapping
- Vertex input mapping order (`attach.rs` and `Renderer::vertex_buffer_layouts` follow the same one):
  1. by name, instance first, then vertex (`position` included)
  2. position at location 0
  3. by the mesh's own buffer location, instance first, then vertex
- Typed Rust values (`f32`, `u32`) must match exactly. Plain numbers take the declared type: floats, or integers when every value is whole and in range. A misfit is an `AttributeMismatch` naming the attribute, the declared type and the value.
- The first shader that reads an untyped attribute fixes its type for the mesh. A shader that declares another type fails to attach with `AttributeConflict`, whatever value the mesh holds now.

Light roots (`light_roots.rs`, `lights.rs`)
- `lights` and `light_grid` are reserved root names in a module with a vertex or fragment entry. `Shader::new` recognizes them before `parse_uniforms`: the read-only storage `Lights` (with an optional `var<storage, read> light_grid: array<u32>`) or the older uniform `LightArray` of `N` lights. Matched roots leave `UniformStorage`, `list_uniforms`, `list_keys` and the storage registry; the renderer fills them.
- A storage `lights` or a `light_grid` with another layout fails with `ShaderError::TypeMismatch` that spells out the layout. A uniform `lights` that matches neither shape stays a plain uniform and logs one warning. Compute-only modules are never checked.
- `set` and `get` on `lights.ambient` reach the shader's ambient term; every other `lights.*` key is `UniformNotFound`.
- A lit shader keeps a weak, deduplicated list of the lights that reach it, in attach order. Passes and scenes hold the lights. Every list or ambient change, every light setter and every light drop bumps the light epoch, which tells the renderer to repack.
- On the uniform light path, `module_for_device` lowers the storage roots, after padding: `lights` becomes a uniform block of the device's light count and `light_grid` a private word. A shader that calls `arrayLength` on a root fails there with `TypeMismatch`. The lowered modules are cached per light count and padding.

Errors & API shape
- Return `ShaderError` from shader APIs; avoid `unwrap` / `expect` / `panic`.
- `set()` is non-blocking (queues last-wins updates); reads may transiently return `ShaderError::Busy` under contention — that is expected.
- Keep public methods thin; delegate to internal state.
