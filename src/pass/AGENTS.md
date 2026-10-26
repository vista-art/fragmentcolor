# AGENTS.md — Pass rules (short)

Role
- Orchestrates one or more shaders and render-time knobs (viewport, clear, compute dispatch, per-pass targets).
- Public methods stay thin and forward to internal objects or helpers.

Targets
- `Pass::set_target(target)` attaches an offscreen color target for intermediate rendering (one per pass; subsequent calls overwrite).
- `Pass::set_depth_target(target)` attaches an offscreen depth target; sample counts must match the color target.
- A pass that draws Models and has no depth target gets a pooled depth attachment from the renderer, matching its color attachment. `set_depth_target` replaces it. Passes without Models draw without depth.
- There is no explicit `present(pass)` API — the last render pass in the executed DAG presents to the final frame by default.

Viewport
- `Pass::set_viewport` refuses a region with a width or height of 0 with `PassError::Viewport`, whatever form it comes in, a region built field by field included. A pass with no viewport draws on the whole target.

Clear color
- `Pass::set_clear_color` hands each channel to wgpu as given. A float target keeps values past 0 to 1, and a unorm target clamps them on the GPU. Never clamp or quantize a clear color on the CPU.
- A NaN or infinite channel returns `PassError::Color` and leaves the pass unchanged. The browser's WebGPU throws a `TypeError` for one.

Compute vs render
- `Pass::is_compute()` reflects whether every attached shader is compute; a pass adopts the kind of the first attached shader.

Mesh handling
- `Pass::add_mesh(mesh)` delegates to the last compatible shader. For per-shader attachment, callers use `Shader::add_mesh(mesh)` directly. `Pass` never owns mesh validation itself.
- Rejecting incompatible meshes is the shader's job; passes propagate the resulting `PassError` unchanged.

Host objects
- FragmentColor holds no host object, such as a Python object or a JS value, and `render` never calls into host code.
- The one exception is a Python `RenderCanvasTarget` (src/target/platform/python.rs). It keeps the canvas, which the RenderCanvas context protocol reads back, and the present methods it was given.
- Don't add another. A host object held in Rust forms a cycle the host's garbage collector can't see, so it never frees either side.
