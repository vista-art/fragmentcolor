# AGENTS.md — Mesh rules (short)

Schema & packing
- Derive vertex / instance schema from the first vertex / instance added; keep the schema stable for the mesh lifetime.
- Pack CPU buffers according to that schema; maintain GPU buffers with caching and dirty flags.

GPU buffers
- `vertex_buffers(device, queue)` must create / update and cache GPU buffers; avoid re-allocating unless a dirty flag demands it.
- Instance buffers are optional; `instance_count` can be overridden to drive non-instanced draws.

Validation contract
- Mesh compatibility is validated by `Shader::add_mesh` at attach time; the renderer performs no runtime validation at draw time.
- Reject invalid attachments with `ShaderError` — never silently accept a mismatched mesh.

Builders & primitives
- Provide idiomatic builders for common shapes (currently `Quad`). Keep them under `mesh/primitives/` and expose via the mesh module re-exports.
