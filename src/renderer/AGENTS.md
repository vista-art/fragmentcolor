# AGENTS.md — Renderer rules (short)

Core invariants
- Draw all meshes attached to a Shader under the same RenderPipeline (per-shader batching).
- Pipeline cache keys are descriptive structs, not tuples:
  - `RenderPipelineKey { shader_hash, color_format, depth_format, sample_count }`
  - `ComputePipelineKey { shader_hash }`
  Do not rely on tuple ordering; extend these structs if new state starts to affect pipeline configuration.
- If a shader has no `@location` vertex inputs, draw a single fullscreen triangle (three built-in verts).
- Vertex layouts are derived once from an authoritative mesh; the renderer does not re-validate per-mesh at draw time.

Render targets & present
- If no per-pass offscreen target is set, render to the provided final frame by default.
- Per-pass targets render intermediate results; the last render pass in the executed DAG becomes the presentation pass automatically (there is no explicit `Frame.present(...)` API).

MSAA & resolves
- Use transient MSAA textures from the texture pool and resolve into the target view; return them to the pool after use.
- Sample count is negotiated per adapter/format via `pick_sample_count` and cached in `RenderContext`.

Bind groups & alignment
- Sort bind group entries by binding index; create empty groups for expected-but-unused groups so indices stay stable.
- Uniforms: align to `device.limits().min_uniform_buffer_offset_alignment`.
- Storage buffers: bind the full buffer (`size: None`) to avoid backend padding surprises.
- Push constants: prefer native push constants when the adapter supports them; fall back to a per-root uniform buffer otherwise (single source of truth in `PushMode`).

Mobile uniffi path
- Methods on `Renderer` that need to be uniffi-exported live in `platform/mobile/`; keep them concrete-typed (uniffi cannot marshal `impl Trait`).
- Platform-specific constructors (e.g. `create_target_ios`) go in `platform/mobile/{ios,android}.rs` and are wrapped by idiomatic Swift/Kotlin extensions in `platforms/{swift,kotlin}/`.

Code structure
- Avoid large nested closures; extract helpers or keep logic linear.
- Keep the public API thin and delegate internally.
- Maintain zero warnings; prefix intentionally unused variables with underscore.
