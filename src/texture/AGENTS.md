# AGENTS.md — Texture & Target rules (short)

Lifecycle & creation
- `TextureTarget` is always created by the `Renderer` (direct construction is crate-private) so usage flags and format negotiation stay correct.
- Prefer the `Renderer::create_texture*` helpers — they wire up the proper `wgpu::TextureUsages`, register the texture with the context, and hand back a typed `Texture` / `TextureId` handle.

Binding & sampling
- Texture views are created per bind. Each texture keeps one sampler built from its `SamplerOptions` and rebuilds it in `set_sampler_options`.
- Bind texture + sampler pairs in consistent groups; fill missing groups with empty bind groups so group indices stay stable.
- A `sampler` binding takes its options from a texture in the same group: the lowest binding between the previous sampler and itself, else the nearest one. `SamplerSources` in `src/renderer/sampler_pairs.rs` owns the rule; the binding's comparison flag wins over the texture's `compare`.
- `UniformData::Texture` carries `TextureMeta` (dim, arrayed, class) so `shader.set("tex", &texture)` only updates the id while preserving the reflected metadata.

Updates & readback
- `Texture::write(&bytes)` uploads a whole-texture frame; `Texture::write_region(&bytes, region)` handles sub-rectangles, layered/3D textures, and explicit source layout. The `region` arg accepts `[w,h]`, `[x,y,w,h]`, `[x,y,z,w,h,d]`, a `ScreenRegion` (its pixel rectangle at `z = 0`, one layer deep), or a `TextureRegion` constructed with `.with_stride(...)` / `.with_rows(...)`.
- `TextureTarget::get_image()` reads back RGBA8 bytes for CI / snapshot tests and returns `TextureError::Readback` when the readback fails. A window target's `get_image()` always returns that error, because a frame on screen can't be read back.

MSAA & resolve
- For MSAA render passes, render to transient MSAA textures from the pool and resolve into the target view; release to the pool after use.
- Sample count must match between color and depth attachments within a pass — `RendererError::DepthSampleCountMismatch` surfaces the violation.
