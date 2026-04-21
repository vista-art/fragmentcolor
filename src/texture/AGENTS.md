# AGENTS.md — Texture & Target rules (short)

Lifecycle & creation
- `TextureTarget` is always created by the `Renderer` (direct construction is crate-private) so usage flags and format negotiation stay correct.
- Prefer the `Renderer::create_texture*` helpers — they wire up the proper `wgpu::TextureUsages`, register the texture with the context, and hand back a typed `Texture` / `TextureId` handle.

Binding & sampling
- Texture views and samplers are (re)created on demand today; descriptor-keyed caches are planned but not required.
- Bind texture + sampler pairs in consistent groups; fill missing groups with empty bind groups so group indices stay stable.
- `UniformData::Texture` carries `TextureMeta` (dim, arrayed, class) so `shader.set("tex", &texture)` only updates the id while preserving the reflected metadata.

Updates & readback
- `Renderer::update_texture` / `update_texture_with` push byte data into an already-registered texture; prefer `TextureWriteOptions::whole()` unless you need a sub-rect.
- `TextureTarget::get_image()` reads back RGBA8 bytes for CI / snapshot tests — window-backed targets return an empty vec because readback is not portable.

MSAA & resolve
- For MSAA render passes, render to transient MSAA textures from the pool and resolve into the target view; release to the pool after use.
- Sample count must match between color and depth attachments within a pass — `RendererError::DepthSampleCountMismatch` surfaces the violation.
