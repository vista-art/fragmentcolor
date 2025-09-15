# Texture feature rollout checklist

Status: WIP

- [ ] Core types
  - [x] Rename internal renderer/texture.rs::Texture -> TextureObject with sampler options (RwLock)
  - [ ] Public API: Texture wrapper (Arc<TextureObject>, u64 handle), simple config methods
  - [ ] Handle registry in RenderContext (DashMap<u64, Arc<TextureObject>>) and helpers

- [ ] Renderer API
  - [ ] create_texture_from_bytes(&[u8]) -> Texture (Rust)
  - [ ] create_texture_from_file(&Path) -> Texture (Rust)
  - [ ] Web: createTextureFromBytes(Uint8Array), fetchTexture(url)
  - [ ] Python: create_texture_from_bytes(bytes), create_texture_from_file(str)

- [ ] Shader UX
  - [ ] UniformData: implement From<&Texture> -> UniformData::Texture(handle)
  - [ ] JS/Python conversions to allow shader.set("key", texture)
  - [ ] naga parsing: detect image/sampler bindings; store in Uniform metadata

- [ ] Renderer bindings and draw
  - [ ] Bind group layout: add Texture and Sampler entries
  - [ ] Render: bind TextureView and Sampler (resolve from handle); fallback texture

- [ ] Docs
  - [ ] docs/api/core/texture/*.md (Texture, set methods, examples)
  - [ ] docs/api/core/renderer/create_texture*.md
  - [ ] docs/api/core/shader/set.md: add texture examples and parity for JS/Py

- [ ] Examples and tests
  - [ ] Rust example: textured quad via fullscreen triangle sampling
  - [ ] Web example: load image URL, set to shader
  - [ ] Python example: load file bytes, set to shader
  - [ ] Unit tests: parse image/sampler, build bind groups, render offscreen

- [ ] Cleanup
  - [ ] TextureError (thiserror); remove Box<dyn Error> in renderer/texture.rs
  - [ ] cargo fmt && cargo clippy --all-targets --all-features -- -D warnings
