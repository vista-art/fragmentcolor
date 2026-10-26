# AGENTS.md — Renderer rules (short)

Core invariants
- Draw all meshes attached to a Shader under the same RenderPipeline (per-shader batching).
- Pipeline cache keys are descriptive structs, not tuples:
  - `RenderPipelineKey { shader_hash, color_format, depth, sample_count, alpha_mode, double_sided, vertex_layouts }`, where `depth` is a `DepthUse` (`src/renderer/depth.rs`)
  - `ComputePipelineKey { shader_hash, bind_group_entries }` (layout entry contents, never resource addresses)
  Do not rely on tuple ordering; extend these structs if new state starts to affect pipeline configuration.
- If a shader has no `@location` vertex inputs, draw a single fullscreen triangle (three built-in verts). A shader that reads `@location` inputs, plain or in a struct argument, and has no mesh returns `ShaderError::InvalidKey` before any pipeline is built.
- Vertex layouts are derived from the shader's first mesh and are part of the pipeline key, since shaders with the same source can feed meshes with different layouts. The renderer does not re-validate per-mesh at draw time.
- A mesh with an empty index list draws nothing. A Model mesh with no visible Model in the pass draws nothing either, which covers a hidden Model and a Model added to another pass through a shared Material. Drawing it would read whatever buffer an earlier mesh left in slot 1.
- When the pipeline reads instance attributes, a mesh draws no more instances than it holds. A larger `set_instance_count` returns `MeshError::InvalidInstance`.

Error scopes & the pipeline cache
- `RenderContext::validate(label, op)` is the one place GPU work runs in a validation error scope. It pops the scope on every path, `op`'s own errors included. On native a captured error returns from that call as `RendererError::ValidationError { label, .. }`.
- Each render and compute pipeline builds inside `validate` and enters its cache through `or_try_insert_with`, so on native a pipeline that failed is never cached and the next frame reports the same error.
- Each frame records its passes and submits them inside one `validate("frame", ..)`. A frame that fails is not presented.
- On wasm a scope resolves after `render` returns. `validate` hands the popped scope to `ErrorSlot::watch` (`late.rs`), whose spawned future stores the first error caught in `RenderContext::late`. The next `render`, `load`, `read_storage` or readback (`read_pixels`, behind `read_texture` and both `get_image`) takes it and returns it before any GPU work, once. A scoped error never reaches the uncaptured handler, so it never logs a `console.error`.
- On wasm a failed pipeline is cached, since its error lands after the insert, and each frame that uses it reports an error again. A check that has to work on the web in the same call runs on the CPU before any wgpu call.

Depth
- `RenderContext::resolve_depth` picks each render pass's depth attachment. A depth target the pass set wins, and its sample count must match the color attachment (`DepthSampleCountMismatch`). Otherwise a pass with Models takes a pooled `Depth32Float` sized and sampled like its color attachment, cleared to 1.0 and discarded at the end. Any other pass draws without depth, in order.
- In a pass that draws Models, a shader that draws none of them (a fullscreen backdrop, an overlay mesh) uses `DepthUse::Ignore`, with the pooled depth or a depth target the pass set: it passes every fragment and writes no depth, so it draws in order and hides no model. In a pass without Models, an explicit depth target makes every shader test depth. Every pipeline in a pass declares the attachment's format, so ignoring depth is a pipeline state, never a missing attachment.
- A render pass takes its size from `TargetFrame::size`, never `Target::size`. A resize can land between acquiring the frame and drawing, and every attachment must match the frame.
- Blend draws test depth and never write it.

Render targets & present
- If no per-pass offscreen target is set, render to the provided final frame by default.
- Per-pass targets render intermediate results; the last render pass in the executed DAG becomes the presentation pass automatically (there is no explicit `present(...)` API).
- A pass viewport goes to wgpu as given, within wgpu's limits, and the scissor rect is the viewport clipped to the pass's color attachment. Output past the attachment is cropped, never squeezed. A viewport that clips to nothing keeps the pass's clear and skips its draws, because Metal can't take an empty scissor.

MSAA & resolves
- Use transient MSAA textures from the texture pool and resolve into the target view. Take them with `Lease::take`, which returns the texture to the pool when the lease drops, error paths included.
- `RenderContext::render` calls `TexturePool::start_render` once when it acquires a frame (compute-only renders don't count), which frees entries no render used for `IDLE_RENDERS` renders, so a target that changed size doesn't keep its old textures.
- Sample count is negotiated per adapter/format via `pick_sample_count` and cached in `RenderContext`.

Bind groups & alignment
- Sort bind group entries by binding index; create empty groups for expected-but-unused groups so indices stay stable.
- Uniforms: align to `device.limits().min_uniform_buffer_offset_alignment`.
- Uniform and storage pools rewind once per `render`, never per pass. `queue.write_buffer` applies at submit, so passes that share a submit need distinct offsets.
- Storage buffers: bind the full buffer (`size: None`) to avoid backend padding surprises.
- Push constants: prefer native push constants when the adapter supports them; fall back to a per-root uniform buffer otherwise (single source of truth in `PushMode`).

Lights (`lights.rs`, `lights/grid.rs`, `light_binning.wgsl`)
- `Renderer::context` picks a `LightPath` once per context from the adapter and device, never from `cfg(wasm)`. `Clustered` needs a backend other than GL, compute shaders, fragment storage, two storage buffers per stage, 64-thread workgroups, 54 workgroups per dimension, 4 KiB of workgroup memory and a 1 MiB storage binding. Every other device, WebGL2 and native GL included, takes `Uniform` with a block of `(min(max_uniform_buffer_binding_size, 64 KiB) - 144) / 64` lights. The path lives on the context, so `RenderPipelineKey` has no light field, and a test that forces a path uses a fresh `Renderer`.
- The context lowers the process-wide light cap (`shader::lower_light_cap`), which `Pass::add` and `Model::attach` check before they change anything. The cap always comes from the device.
- `prepare_lights` runs in `process_render_pass` after `build_pass_draws` and before `begin_render_pass`, and nowhere else. It checks each lit shader against the context's cap first, so a shader past it fails with `RendererError::PassError(LightCapReached)` before any light buffer is touched. It records the binning compute pass on the frame encoder. It touches buffers only, so the Apple compute split doesn't apply.
- Lit shaders on a pass share a `LightSet` when their light lists, ambient terms and root shapes match, keyed by `(pass id, set key)`, with the shader in the key when the pass has no `Camera`. Sets stay out of `storage_registry` and the per-frame pools, and a set unused for `IDLE_RENDERS` renders is freed. Each set is prepared at most once per render: it repacks when the light epoch moved, uploads only bytes that changed, and rebins only when a bounding sphere, the binning matrix or a buffer changed. A still scene dispatches once, and a pass with no binned light never dispatches.
- The bind loop binds each lit shader's light buffers whole, with `size: None`, before the reflected uniforms. Light root layout entries are visible to `FRAGMENT`, plus `VERTEX` when the vertex entry reads a root, and the push-constant fallback group sits past the highest reflected or light group.
- The binning pipeline builds once per context inside `validate`; a native failure isn't cached. CPU-side math in `lights/grid.rs` mirrors the WGSL, and `constants_match` keeps the grid constants and struct offsets of the three WGSL files in step with Rust.
- Mesh buffers belong to the device that drew the mesh first, so a GPU test that compares renderers builds a new pass for each one.

Mobile uniffi path
- Methods on `Renderer` that need to be uniffi-exported live in `platform/mobile/`; keep them concrete-typed (uniffi cannot marshal `impl Trait`).
- Platform-specific constructors (e.g. `create_target_ios`) go in `platform/mobile/{ios,android}.rs` and are wrapped by idiomatic Swift/Kotlin extensions in `platforms/{swift,kotlin}/`.

Code structure
- Avoid large nested closures; extract helpers or keep logic linear.
- Keep the public API thin and delegate internally.
- Maintain zero warnings; prefix intentionally unused variables with underscore.
