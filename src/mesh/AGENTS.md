# AGENTS.md — Mesh rules (short)

Schema & packing
- Derive the vertex / instance field list from the first vertex / instance added. Field names, order and sizes stay stable for the mesh lifetime, except that `clear_instances` drops an instance layout built from instances (a `Model`'s declared columns stay) and the next instance sets it again.
- A typed value (a Rust `f32` or `u32`) fixes its field's format. An untyped field (plain numbers, the form every binding sends) takes the format of the first shader that reads it, for the mesh's lifetime, and packs as floats until one does. A recorded type wins over a later typed value, which then fails the attach and the pack. Typing never moves an offset or the stride, since every component is 4 bytes.
- `ensure_packed` derives and stores a missing schema under that schema's lock, and `types` is a leaf lock, so an attach running at the same time can't leave a schema derived from old types.
- `Vertex` and `Instance` values never change at attach: the same vertex can sit in two meshes whose shaders type it differently. The recorded types live on the mesh (`MeshObject.types`).
- Pack at render (`ensure_packed`), converting each value to its field's format. Values added after the attach are checked there and fail with `MeshError`.
- Maintain GPU buffers with caching and dirty flags. Recording a new type resets the schema that lists it, marks its stream dirty and invalidates the cache; an attach that records nothing keeps the packed bytes and GPU buffers.

GPU buffers
- `vertex_buffers(context)` must create / update and cache GPU buffers; avoid re-allocating unless a dirty flag demands it.
- The buffers belong to one `RenderContext` at a time (`GpuStreams::owner`, its `id`). Another context gets buffers of its own, uploaded from the packed bytes, so a mesh draws with any `Renderer`.
- Instance buffers are optional; `instance_count` can be overridden to drive non-instanced draws.

Validation contract
- Mesh compatibility is validated by `Shader::add_mesh` at attach time. It checks every vertex and instance value and records types only when it succeeds; `Shader::validate_mesh` records nothing.
- Reject invalid attachments with `ShaderError` (`AttributeMismatch`, `AttributeConflict`, `InvalidKey`). Never silently accept a mismatched mesh.

Builders & primitives
- Provide idiomatic builders for common shapes (currently `Quad`). Keep them under `mesh/primitives/` and expose via the mesh module re-exports.
