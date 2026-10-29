# AGENTS.md — Shader rules (short)

Ownership & validation
- `Shader` owns `Mesh` attachments; validate at attach time and reject incompatible meshes with a typed `ShaderError`.
- No invalid mesh may be attached to a shader under any circumstance.
- If the shader has no `@location` vertex inputs (fullscreen / builtin-only), attaching a mesh must be rejected.
- `reflect_vertex_inputs` is the one list of vertex inputs. It holds each `@location` argument of the vertex entry point and each `@location` member of a struct argument, under the member's name, so `bind_mesh`, the renderer's vertex layouts and its no-mesh check read struct members as they read arguments. A struct of built-ins only adds nothing and stays a fullscreen input.
- Attach checks every vertex and instance value, not only the first, and each one must hold every attribute the shader reads. A failed attach records nothing on the mesh, so `Pass::add_mesh` can try the next shader.

Constructors
- `Shader::new(source: &str) -> Result<Self, ShaderError>` — WGSL or GLSL source; the core Rust entry point.
- `Shader::from_mesh(mesh: &Mesh) -> Self` — infers a minimal shader from a mesh (returns a default shader on failure and logs the error; never panics).
- `Shader::from_vertex(v: &Vertex) -> Self` — same idea from a single vertex (pos dim + common properties).
- Mobile uniffi: `Shader::new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError>` in `platform/mobile.rs` (owned `String` + `Arc<Self>` as uniffi requires). Swift / Kotlin extensions re-expose it as `Shader(source)`.

Creation checks (`Shader::wgsl`, which every constructor reaches)
- Parse and validate with `create_validator` / `create_parser`: the naga capabilities of `CANDIDATES` (`renderer/platform/all.rs`), the features a device may open with. A shader that needs another feature (`f16`, subgroups, `texture_external`, texture atomics) fails here, never at render.
- `parse_uniforms` refuses two globals on one `@group`/`@binding` (`claim_slot`) and writable storage that a `@vertex` entry point uses. It records each global's `Uniform::stages` from naga's `ModuleInfo`; the renderer makes each binding visible to those stages, and a global no entry point uses to the compute stage.
- `immediates::check_immediates` lowers the immediate roots to uniforms on a copy and validates it, since the web and devices without immediates bind them that way. `lower_immediates` is the one rewrite, shared with the renderer.
- `layouts::write_layouts` writes the WGSL the browser's WebGPU gets for a module whose buffer structs set `@size`/`@align`: naga's own writer output, with each pinned struct renamed to a name naga's namer keeps and a `@size` on each member but a runtime-sized array. It reads the text back and refuses a struct whose layout moved. Only the renderer calls it, on the browser's WebGPU, so `Shader::new` never refuses a shader because of the web's text. `list_explicit` names those structs for the GL refusal.
- `check_samplers` refuses a filtering `sampler` that samples a depth texture, which the web refuses. A light root still claims its slot in `parse_uniforms`, so a global on it fails as two globals on one slot do.
- Every texture setter for a key takes a new stamp (`stamp_texture`). A pending upload carries its stamp, and `upload_pending` and `requeue_textures` drop one a newer setter call replaced, so the latest call wins across an `.await`. Every texture write and stamp holds `texture_writes`, and `set_latest` checks an upload's stamp and binds its texture under it, so a setter call on another thread lands before or after both, never between them.
- `reflect_vertex_inputs` runs once at creation and refuses two inputs with one name.
- Raw bytes for a storage or immediate root must be exactly its size (`UniformInput::resolve`, `set_storage_bytes`, `set_push_bytes`).
- Array elements get no keys of their own; `locate_key` resolves `u.items[2].v` by walking the shape.

Reflection & mapping
- Vertex input mapping order (`attach.rs` and `Renderer::vertex_buffer_layouts` follow the same one):
  1. by name, instance first, then vertex (`position` included)
  2. position at location 0
  3. by the mesh's own buffer location, instance first, then vertex
- Typed Rust values (`f32`, `u32`) must match exactly. Plain numbers take the declared type: floats, or integers when every value is whole and in range. A misfit is an `AttributeMismatch` naming the attribute, the declared type and the value.
- The first shader that reads an untyped attribute fixes its type for the mesh. A shader that declares another type fails to attach with `AttributeConflict`, whatever value the mesh holds now.
- Every pipeline compiles from a naga module. On the browser's WebGPU, wgpu writes it back to WGSL, which drops `@size` and `@align`, so a module with explicit layouts goes as `write_layouts` text there. The GL backends write a std140 block member by member and refuse such a layout. Built-in WGSL spells its padding out as members, as `PbrMaterial` and `TextureUv` do, which every backend keeps, and `pbr_layout_survives_the_web_round_trip` checks the PBR layouts through the WGSL round trip.
- On a backend that pads uniform bindings to 16 bytes (GL, WebGL2), `pad_uniform_blocks` pads immediate roots too, and the renderer always lowers them there, so a lowered root binds a multiple of 16 bytes.

Light roots (`light_roots.rs`, `lights.rs`)
- `lights` and `light_grid` are reserved root names in a module with a vertex or fragment entry. `Shader::new` recognizes them before `parse_uniforms`: the read-only storage `Lights` (with an optional `var<storage, read> light_grid: array<u32>`) or the older uniform `LightArray` of `N` lights. Matched roots leave `UniformStorage`, `list_uniforms`, `list_keys` and the storage registry; the renderer fills them.
- A storage `lights` or a `light_grid` with another layout fails with `ShaderError::TypeMismatch` that spells out the layout. A uniform `lights` that matches neither shape stays a plain uniform and logs one warning. Compute-only modules are never checked.
- `set` and `get` on `lights.ambient` reach the shader's ambient term; every other `lights.*` key is `UniformNotFound`.
- A lit shader keeps a weak, deduplicated list of the lights that reach it, in attach order. Passes and scenes hold the lights. Every list or ambient change, every light setter and every light drop bumps the light epoch, which tells the renderer to repack.
- On the uniform light path, `module_for_device` lowers the storage roots, after padding: `lights` becomes a uniform block of the device's light count and `light_grid` a private word. A shader that calls `arrayLength` on a root fails there with `TypeMismatch`. The lowered modules are cached per light count and padding.

Errors & API shape
- Return `ShaderError` from shader APIs; avoid `unwrap` / `expect` / `panic`.
- `set()` is non-blocking (queues last-wins updates); reads may transiently return `ShaderError::Busy` under contention — that is expected.
- Keep public methods thin; delegate to internal state.
