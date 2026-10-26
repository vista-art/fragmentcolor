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

Mobile uniffi path
- Methods on `Renderer` that need to be uniffi-exported live in `platform/mobile/`; keep them concrete-typed (uniffi cannot marshal `impl Trait`).
- Platform-specific constructors (e.g. `create_target_ios`) go in `platform/mobile/{ios,android}.rs` and are wrapped by idiomatic Swift/Kotlin extensions in `platforms/{swift,kotlin}/`.

Code structure
- Avoid large nested closures; extract helpers or keep logic linear.
- Keep the public API thin and delegate internally.
- Maintain zero warnings; prefix intentionally unused variables with underscore.
