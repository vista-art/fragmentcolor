# AGENTS.md — Pass rules (short)

Role
- Orchestrates one or more shaders and render-time knobs (viewport, clear, compute dispatch, per-pass targets).
- Public methods stay thin and forward to internal objects or helpers.

Targets
- `Pass::add_target(target)` attaches an offscreen color target for intermediate rendering.
- `Pass::add_depth_target(target)` attaches an offscreen depth target; sample counts must match the color target.
- There is no explicit `Frame.present(pass)` — the last render pass in the executed DAG presents to the final frame by default.

Compute vs render
- `Pass::is_compute()` reflects whether every attached shader is compute; a pass adopts the kind of the first attached shader.

Mesh handling
- `Pass::add_mesh(mesh)` and `Pass::add_mesh_to_shader(mesh, shader)` delegate to the relevant shader; `Pass` never owns mesh validation itself.
- Rejecting incompatible meshes is the shader's job; passes propagate the resulting `PassError` unchanged.
