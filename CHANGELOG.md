# Changelog

## 0.12.0: 3D scenes: glTF loading, PBR materials, and a texture pipeline

Large release that implements support for 3D assets.

A top-level `Scene` graph, a glTF 2.0 loader (`Scene::load`), full PBR metallic-roughness materials (texture sampling, alpha modes, tangent-space normal mapping, `KHR_texture_transform`), a unified `Light` type, and `Camera` / `Model` as first-class scene objects. Underneath, texture creation moved off the main thread with KTX2 + 16-bit format support, the public API thinned to a single transport per operation, and the texture-related public surface got a structural cleanup before tagging.

### `Vertex::pbr` convenience helper

Quality-of-life cleanup after the v0.11.2 PBR vertex layout grew to five optional attributes (`NORMAL`, `UV0`, `COLOR0`, `UV1`, `TANGENT`). `Vertex::pbr(position)` constructs a vertex with all five seeded to neutral identity defaults; chain `.set(...)` to override the ones the caller actually has data for. Mechanical-but-significant doctest cleanup: 17 files trimmed of the 5-line `Vertex::new(...).set(NORMAL,...).set(UV0,...).set(COLOR0,...).set(UV1,...).set(TANGENT,...)` chain.

- [x] **`Vertex::pbr<P: IntoVertexPositionFull>(position)`** in `src/mesh/vertex.rs`. Layers default `NORMAL = [0,0,1]`, `UV0 = [0,0]`, `COLOR0 = [1,1,1,1]`, `UV1 = [0,0]`, `TANGENT = [1,0,0,1]` onto a fresh `Vertex::new` via the existing `.set(...)` chain so re-overrides via `.set(Vertex::NORMAL, real)` work as expected (the prop_location stays put on second set, only the value updates).
- [x] **Cross-platform bindings:** Python (`Vertex.pbr([x, y, z])`), JS / wasm-bindgen (`Vertex.pbr([x, y, z])`), Swift / Kotlin via uniffi (`Vertex.pbr([x, y, z])` accepting `Vec<f32>` of length 2 / 3 / 4 like `Vertex.new`).
- [x] **Bulk refactor** of 17 files (6 internal test helpers + the `model_pbr_triangle` example + 10 doctest examples under `docs/api/`) to use `Vertex::pbr(p).set(Vertex::UV0, uv)` instead of the 5-line default chain. Net diff: -150 lines.
- [x] **Verification:** 255 lib tests + 153 doctests passing (`Vertex::pbr` adds one doctest); `gltf_scene` and `model_pbr_triangle` examples render.

### Wild-glTF correctness: seven of seven gaps closed

This session closed all seven "wild glTF" correctness gaps flagged after the loader landed. The renderer now produces visually-correct output for the full glTF 2.0 PBR-MR surface, including translucent materials, the long-standing deferral from the prior commit batch. Opaque + Mask + Blend all interoperate inside one Pass without caller-side sort management.

Closed in this session:
1. **Multi-light + ambient** (prior commit `09a79bdb`)
2. **Face-normal fallback + sampler options** (commit `3ecd35aa`)
3. **`COLOR_0` + `TEXCOORD_1` vertex inputs** (commit `97f281d0`)
4. **`KHR_texture_transform`** (commit `12feec48`)
5. **Tangent-space normal mapping** (commit `06639a6e`)
6. **Transparency depth-sort for `alpha_mode: Blend`** (this commit)

### Transparency depth-sort for `alpha_mode: Blend`

Seventh "wild glTF" gap closed. `Material::alpha_mode(AlphaMode::Blend)` Models now over-blend in the correct order regardless of `Pass::add(&model)` insertion order. The renderer partitions each Pass's Model entries by alpha mode, batches opaque/mask draws the way it always has (one instanced draw per `(shader, mesh)` group, so the auto-instancing fast path stays intact), and submits the translucent draws one-by-one, sorted back-to-front by eye-space Z. Translucent glTFs (frosted glass, smoke, fades, decals that need soft edges) compose correctly today without any caller-side sort logic.

- [x] **`PassObject.camera_snapshot: RwLock<Option<CameraSnapshot>>`** in `src/pass/mod.rs`. Caches the world-space eye position + view matrix when `Camera::attach` runs; refreshed by `Camera::look_at` through the existing `Camera::propagate` hook. The renderer reads it for the sort instead of going through `Shader::get("camera.position")`. That path only worked for shaders that happened to expose those uniforms under canonical names. `None` (no Camera attached to the pass) falls back to insertion-order, which is correct for an opaque-only Pass and the best the renderer can do for a translucent one without world-space depth.
- [x] **`Camera.attached_passes: RwLock<Vec<Weak<PassObject>>>`** in `src/scene/camera.rs`. Held weakly so a dropped Pass doesn't keep the Camera-side handle alive, with the same share semantics as the existing `attached: Vec<Weak<ShaderObject>>` for shader-uniform propagation.
- [x] **`Renderer::build_pass_draws`** replaces `build_model_instance_overrides` in `src/renderer/mod.rs`. Walks `pass.model_entries`, reads each entry's shader's `alpha_mode`, partitions into:
  - `opaque_overrides: HashMap<(shader_ptr, mesh_ptr), (wgpu::Buffer, u32)>`, one batched instance buffer per `(shader, mesh)` pair, packing every queued model matrix; the auto-instancing path.
  - `blend_draws: Vec<BlendDraw>`, one `(shader, mesh, transform, eye_z, instance_buffer)` tuple per blend entry; **sorted ascending by eye-Z** (right-handed view space: smaller Z is farther from the camera, drawn first).
- [x] **Per-shader dispatch branch in `process_render_pass`**. After the existing pipeline + bind-group setup, the draw-decision switches on the shader's `alpha_mode`: Opaque/Mask takes the existing batched path; Blend walks the sorted `blend_draws` for this shader and issues one `draw_indexed(.., 0..1)` per entry with the entry's single-instance vertex buffer at slot 1. Cross-Material interleaving falls back to per-shader sort. Most translucent scenes use one Material shared across many Models, which is order-correct; multi-Material global interleave rides on a follow-up if it ever bites.
- [x] **`AlphaMode` doc-comment + `docs/api/scene/material/alpha_mode.md`** updated to describe the new automatic-sort behaviour. Removes the "sort back-to-front yourself" caveat; callers don't need to.
- [x] **Smoke example** `examples/rust/examples/transparent_quads.rs` renders three stacked colored quads with `alpha_mode: Blend` at distinct Z depths, added to the Scene in deliberately-wrong order. The output frame's 12,100 non-empty pixels are all blended (none reach saturation in any RGB channel), confirming over-blend is happening through the sort path. Run: `cargo run -p fce --example transparent_quads`.
- [x] **Tests:** `pass_add_stamps_camera_snapshot` (verifies `Camera::attach` writes the snapshot and `Camera::look_at` refreshes it through `propagate`) plus two end-to-end render tests in `tests/blend_transparency.rs` (`renders_two_blend_quads_at_different_depths` covers the blue-over-red dominance after sort; `opaque_and_blend_in_same_pass_render_without_panic` covers the partition path). 256 lib tests + 152 doctests passing.

### Tangent-space normal mapping

Sixth "wild glTF" gap closed. Normal maps now apply correctly to non-Z-facing surfaces via a full TBN transform, replacing the placeholder additive perturbation that worked only on geometry facing the camera. Tangents come from the glTF `TANGENT` accessor when present, with a fixed-axis fallback for assets that omit them (MikkTSpace tangent generation is a follow-up).

- [x] **WGSL `vs_main` input** `@location(9) tangent: vec4<f32>` (glTF spec layout: `.xyz` = object-space tangent direction, `.w` = ±1 bitangent sign for TBN handedness). Transformed by the model matrix's upper-3×3 (tangents are direction vectors, no inverse-transpose) and forwarded to `fs_main` via `VsOut.world_tangent`.
- [x] **`fs_main` TBN transform** replaces the placeholder normal perturbation. Decodes the tangent-space normal from `[0, 1]` to `[-1, 1]`, scales the XY by `material.normal_scale` (glTF spec), then rotates into world space through `T·tn.x + B·tn.y + N·tn.z` where `B = cross(N, T) * tangent.w`. The default 1×1 normal-map encoding `(128, 128, 255)` decodes to tangent-space `(0, 0, 1)`, so the lit normal collapses to geometric `N` and unset normal maps don't perturb shading. Normal maps now light correctly on non-Z-facing surfaces.
- [x] **Loader:** `reader.read_tangents()` extracts `TANGENT` (decoded to `[f32; 4]` via the gltf crate's typed accessor); fallback is `[1, 0, 0, 1]` (T = +X, sign = +1) for assets that omit tangents. Documented in-line as the MikkTSpace-tangent follow-up.
- [x] **All PBR vertex call sites updated** to seed `TANGENT` defaults: `src/scene/{light,camera,mod,scene}.rs`, `src/material/mod.rs`, the `model_pbr_triangle` example, and 20 doctest examples under `docs/api/`.
- [x] **Tests:** 255 lib tests + 152 doctests passing; the `gltf_scene` and `model_pbr_triangle` examples render with the new vertex layout.

### Material: `KHR_texture_transform` (global UV transform)

Fifth "wild glTF" gap closed. `Material::uv_transform(offset, scale, rotation)` lets callers tile, rotate, or offset every texture sample with a single setter; the loader promotes a glTF `KHR_texture_transform` extension on the base-color slot to the same uniform automatically. Today the transform is global to a Material (one transform for all five maps); per-map transforms, and the per-map `texCoord` selector that picks `UV0` vs `UV1`, are a follow-up. The global path covers the most common usage of the extension losslessly (KHR_texture_transform is usually applied to `base_color` alone).

- [x] **`Material::uv_transform([f32; 2] offset, [f32; 2] scale, f32 rotation)`** builder method. Sets three uniforms (`material.uv_offset`, `material.uv_scale`, `material.uv_rotation`) routed through the existing PBR Material uniform binding. `Material::apply_defaults` seeds identity (`scale = [1,1]`, `rotation = 0`, `offset = [0,0]`) so unset materials sample textures untransformed.
- [x] **WGSL `fs_main`** applies the transform in scale → rotate → offset order (matches the glTF spec's composition) once per fragment, then feeds the result to every `textureSample` call. The matrix math is cheap (a sin/cos pair + two muls + an add), well within fragment-shader budget.
- [x] **Loader:** `info.texture_transform()` on the base-color `TextureInfo` lifts the extension's offset / scale / rotation into the Material via `uv_transform`. Other-slot transforms are ignored for now; the loader will warn in a follow-up commit when an asset has divergent per-map transforms.
- [x] **`gltf` crate feature** `KHR_texture_transform` enabled in Cargo.toml alongside `KHR_lights_punctual`.
- [x] **Cross-platform bindings:** Python (`material.uv_transform(offset, scale, rotation)`), JS / wasm-bindgen (`material.uvTransform(offset, scale, rotation)`), Swift / Kotlin via uniffi (`material.uvTransform(offset, scale, rotation)`). All take length-2 arrays and validate.
- [x] **Tests:** 255 lib tests + 152 doctests passing (`uv_transform` adds one doctest); `gltf_scene` example renders.

### PBR shader: vertex colors (`COLOR_0`) + second UV set (`TEXCOORD_1`)

Two more "wild glTF" gaps closed: per-vertex tinting via `COLOR_0` and the secondary UV set used by maps that opt out of `TEXCOORD_0`. Both attributes become required PBR vertex inputs so the renderer's pipeline layout is stable across meshes; defaults (`[1,1,1,1]` for color, `[0,0]` for uv1) preserve behaviour for callers that don't supply them.

- [x] **WGSL `vs_main` inputs:** `@location(7) color0: vec4<f32>` and `@location(8) uv1: vec2<f32>`. Pass through to `fs_main` via `VsOut.vertex_color` and `VsOut.uv1`. `fs_main` multiplies `albedo *= in.vertex_color.rgb` and `alpha *= in.vertex_color.a` so the existing factor × map product is preserved when vertex color is white. `uv1` is plumbed for the upcoming `KHR_texture_transform` + per-map `texCoord` selector work (the shader still samples every map from `uv0` today).
- [x] **Loader:** `reader.read_colors(0)` extracts `COLOR_0` (decoded to `[f32; 4]` via the gltf crate's typed accessor); `reader.read_tex_coords(1)` extracts `TEXCOORD_1`. Both fall back to defaults when the primitive omits them, and the vertex always carries the four-attribute set so the PBR pipeline layout stays consistent across glTF assets.
- [x] **Bind-point matcher rewrite.** Adding two new vertex attributes exposed a long-standing bug in `Renderer::vertex_buffer_layouts` and `Shader::validate_mesh`: both prioritized location-based matching, which mis-routed shader inputs once the mesh's auto-incremented vertex locations collided with the shader's fixed instance-attribute locations (`model_0..model_3` at 3..6). Both matchers now prefer name match first (`position` / `model_*` / `color0` / `uv1` / etc.) and fall back to location only for unnamed slots. Unblocks every future vertex-attribute addition.
- [x] **All PBR vertex call sites updated** to seed `COLOR0` and `UV1` defaults: `src/scene/light.rs`, `src/scene/camera.rs`, `src/scene/mod.rs`, `src/scene/scene.rs`, `src/material/mod.rs`, `examples/rust/examples/model_pbr_triangle.rs`, plus 20 doctest examples under `docs/api/`.
- [x] **Tests:** existing 255 lib tests + 151 doctests all pass; the `gltf_scene` and `model_pbr_triangle` examples render with the new vertex layout.

### glTF loader: face-normal fallback + sampler options

Two of the seven "wild glTF" correctness gaps closed. POSITION-only assets now smooth-shade correctly instead of getting the +Z placeholder normal; per-texture sampler state from glTF (wrap modes + magFilter) flows into FragmentColor's `SamplerOptions` instead of taking the renderer default.

- [x] **Face-normal computation** in `src/scene/loader.rs`. When a glTF primitive omits `NORMAL`, the loader walks every triangle, accumulates the un-normalized cross product into each vertex slot (area-weighted contribution by construction), and normalizes. Falls back to `+Z` only for vertices no triangle touches. Handles both indexed and sequential-triplet primitive modes. Degenerate triangles (zero-area) contribute zero and don't poison neighbours.
- [x] **`map_sampler_options(&gltf::Sampler)`** translates glTF's `wrapS` / `wrapT` (`REPEAT` / `MIRRORED_REPEAT` / `CLAMP_TO_EDGE`) into `SamplerOptions::repeat_*`, and `magFilter` / `minFilter` (`NEAREST` / `LINEAR` variants) into `smooth`. `MIRRORED_REPEAT` collapses to `REPEAT` (FragmentColor's sampler doesn't expose mirror yet); mipmap-filter variants of `minFilter` collapse to their base filter, since the upload path runs its own mipmap-chain decision based on `options.mipmaps`.
- [x] **`image_to_texture_input`** plumbs the sampler options through `TextureOptions { sampler, .. }` so the renderer's lazy upload path picks them up; the Material's slot-format hint (sRGB vs linear) still wins over the default. Tiled textures repeat instead of clamp; pixel-art / texel-art assets keep their nearest filtering instead of getting smoothed.
- [x] **Tests:** `compute_vertex_normals_indexed_yz_face` (YZ-plane triangle → +X normal, verifies the computation differs from the +Z placeholder), `_non_indexed_walks_triplets` (covers the unindexed branch), `_averages_shared_vertex` (shared vertex across two faces normalizes correctly), `map_sampler_options_handles_repeat_and_nearest` (REPEAT + NEAREST round-trip from glTF JSON). 255 lib tests + 151 doctests passing.

### Multi-light + ambient

Closes the placeholder "multi-light scenes will iterate this binding in a follow-up" item from the prior commit. The PBR shader's lighting binding moves from a single `Light` to a `LightArray { count, ambient, lights: array<Light, 8> }`; `fs_main` loops over `lights.count`, sums per-light contributions, and adds `albedo * lights.ambient` on top. Removes the silent downgrade for any glTF or hand-built scene with more than one light.

- [x] **`LightArray` uniform** at `@group(0) @binding(1)` in `src/material/pbr_main.wgsl`. `PBR_MAX_LIGHTS = 8` matches `crate::scene::light::PBR_MAX_LIGHTS` (the Rust-side const lives in `src/scene/light.rs`). 1 u32 count + vec3 ambient + 8 × Light(64) = 544 bytes, well within wgpu's per-binding uniform cap. Raise both consts together if a scene needs more.
- [x] **`fs_main` accumulation loop.** Per-fragment dispatches on `lt.kind` for every active light (directional / point / spot), accumulates Cook-Torrance + GGX shading into `lit_total`, and adds the scene-wide `lights.ambient` term once. The hardcoded `albedo * 0.03` ambient from the prior shader becomes `albedo * lights.ambient`, with `[0.03, 0.03, 0.03]` seeded as the default so existing scenes render identically.
- [x] **Per-shader slot allocation.** `ShaderObject.user_lights_attached: RwLock<u32>` tracks how many user-supplied Lights have absorbed into the Shader; `Light::apply_to_shader` claims the next free slot, writes its fields under `lights.lights[i].*`, and bumps `lights.count`. Material::pbr seeds slot 0 with a dim directional default so the first user Light *overwrites* slot 0 (not "Light + placeholder"). Subsequent Lights take slots 1, 2, … up to `PBR_MAX_LIGHTS`; the cap warns via `log::warn!` and drops the overflow rather than panicking.
- [x] **Dedup on Pass replay.** Lights stash `(Weak<ShaderObject>, slot: u32)` instead of just the shader handle. `apply_to_shader` checks whether the Light is already absorbed by the shader: yes → rewrite at the existing slot, no → claim a new one. Without this, `Pass::add_model` triggering replay of every `scene_object` would consume a new light slot per replay and cap out after a few Model adds.
- [x] **`Scene::ambient([r, g, b]) -> &Self`.** Sets `lights.ambient` on every shader the scene currently sees AND stashes the value on `SceneInner` so future `Scene::add` calls re-stamp it onto incoming shaders. Lets callers configure scene-wide ambient before or after adding models; either ordering works. Default Material seeding remains `[0.03, 0.03, 0.03]`; `Scene::ambient` overrides per-scene.
- [x] **Shader storage index expansion.** `UniformStorage::add_uniform` now recursively expands `array<struct>` fields into per-element keys, so `Shader::get("lights.lights[3].intensity")` resolves via the fast-path index. Reads and writes both work for any nested array-of-struct uniform, not just the lighting binding.
- [x] **Cross-platform bindings.** `Scene::ambient` exposed on Python (`scene.ambient([r,g,b])`), JS / wasm-bindgen (`scene.ambient([r,g,b])` taking `Vec<f32>` of length 3), Swift / Kotlin via uniffi (`scene.ambient([r,g,b])`).
- [x] **Tests:** `two_lights_take_distinct_slots` (verifies slot 0/1 + `lights.count == 2`), `light_attached_twice_keeps_one_slot` (dedup-on-replay test), `ambient_default_seeds_to_dim_grey`. All five existing single-light tests updated to read via `lights.lights[0].*` and continue to pass. 251 lib tests + 151 doctests passing.

### `Scene::load` + glTF 2.0 loader

Closes the v0.11.2 wishlist item "glTF loader". `Scene::load(impl Into<SceneSource>)` is the format-tagged entry; today the only variant is `Gltf`, covering both `.gltf` JSON (with external buffers + images) and `.glb` binary containers (file path or in-memory bytes). The shape is deliberately analogous to `Material::pbr()` and `Camera::perspective(...)`: sync return, no `Renderer` argument, no `await`. Texture inputs the parser produces flow through the new pending-texture pipeline (see "Material: lazy texture sources" below), so the loader stays clean of GPU dependencies.

- [x] **`SceneSource` enum** at `src/scene/loader.rs`. Single `Gltf(GltfSource)` variant for now; `SceneSource::gltf(impl Into<GltfSource>)` is the typed constructor. `GltfSource = Path(PathBuf) | Bytes(Vec<u8>)`, with `From` impls for `&str` / `&Path` / `PathBuf` / `Vec<u8>` / `&[u8]`. The enum exists so future formats (USD, FBX, …) slot in as new variants without disturbing the public method.
- [x] **`SceneLoadError`** wraps the upstream `gltf::Error` via `#[from]`, plus `Material(ShaderError)` for downstream PBR construction failures and `Invalid(String)` for typed-data-but-semantically-wrong inputs.
- [x] **`gltf` crate** added as a workspace dep with the `KHR_lights_punctual` feature enabled (lights extension is the only opt-in we rely on; `import` and `utils` are defaults).
- [x] **Loader coverage:** mesh primitives (`POSITION`, optional `NORMAL`, optional `TEXCOORD_0`, optional indices via `read_indices`); PBR-MR materials (all five texture slots, `base_color`/`metallic`/`roughness`/`emissive`/`alpha_cutoff` factors, `AlphaMode` mapping, `doubleSided`, `normal_scale`, `occlusion_strength`); per-node transforms flattened into Model matrices (handles both `Matrix` and decomposed `TRS` cases); glTF camera nodes (perspective + orthographic, eye/target/up derived from the world matrix); `KHR_lights_punctual` lights (directional / point / spot, using the new variants from the prior commit). Missing NORMAL / UV0 attributes are filled with safe placeholders so POSITION-only assets still render with valid lighting math.
- [x] **`Scene::load` cross-language bindings.** Python (`Scene.load("path.gltf")` or `Scene.load(bytes)`, auto-detecting string vs bytes), JS / wasm-bindgen (`Scene.load(stringOrUint8Array)`), Swift / Kotlin via uniffi (`Scene.load(path)`; bytes-via-FFI deferred; mobile callers fetch to disk or use the Rust API directly).
- [x] **Docs:** `docs/api/scene/scene/load.md` (canonical) + `hidden/load_bytes_mobile.md` retired in favor of a single `load(path)` on mobile.
- [x] **Tests:** `load_minimal_triangle_glb_returns_scene_with_one_model` constructs a valid `.glb` in memory (`build_minimal_triangle_glb` helper, single triangle, POSITION only) and asserts `scene.passes()` returns one default Pass with one Model entry. `load_falls_back_through_into_for_path_inputs` exercises the `&str → GltfSource → SceneSource` chain. 248 lib tests + 150 doctests passing.
- [x] **Example:** `examples/rust/examples/gltf_scene.rs` builds a minimal `.glb` in memory, hands it to `Scene::load`, and renders the result through `renderer.render(&scene, &target)` with the renderer's default Camera + Light injected at draw time. Single self-contained file; no external asset needed.

### Material: lazy texture sources + `Renderer::load`

Lays the API the glTF loader hangs off without breaking the "Material doesn't see a `Renderer`" contract. Material's five `*_texture` setters graduate from `&Texture` (eager, already-uploaded) to `impl Into<TextureInput>`, the same unified transport `Renderer::create_texture` accepts. Pre-built `Texture` handles still work via the existing `From<&Texture> for TextureInput`; raw bytes, paths, URLs, and `DynamicImage` values land on a per-Shader *pending* list. The renderer drains pending uploads on the first render against a renderable, so the loader-built Scene rounds-trips through `renderer.render(&scene, &target)` without a separate upload step.

- [x] **`ShaderObject::pending_textures`**: a `RwLock<Vec<PendingTexture>>` slot that receives the lazy queue. New methods `queue_or_set_texture(key, input)` (immediate set on `TextureData::CloneOf`, queue otherwise) and `drain_pending_textures()` (swap-and-return). Shader is the natural home: the renderer iterates `pass.shaders` at draw time, so traversal stays linear.
- [x] **Material `*_texture(impl Into<TextureInput>)`.** Each setter routes through `queue_texture_or_warn(...)` and injects the glTF-correct color space hint when the caller didn't pick a non-default `TextureFormat`: `base_color` and `emissive` → `Rgba8UnormSrgb`; `metallic_roughness`, `normal`, `occlusion` → `Rgba8Unorm`. Override with `(source, TextureFormat::...)` if needed.
- [x] **`Renderer::load(impl Renderable) async`**: public eager-prewarm surface. Walks every Shader the renderable visits, drains pending, calls `create_texture` for each, writes the resulting `UniformData::Texture(meta)` under the queued key. Cross-platform parity: `load` exposed on Python (`load(renderable)` sync via `pollster::block_on` per FFI convention), JS / wasm-bindgen (`load(renderable)` async returning `Promise<void>`), Swift / Kotlin via uniffi (`load(renderable)` async).
- [x] **`Renderer::render` auto-drain.** On native, `render` calls `futures::executor::block_on(self.load(renderable))?` before touching the GPU. Cheap when nothing is pending (the future doesn't await), correct when something is. On wasm there's no blocking executor; the wasm `render_js` shim is responsible for `await renderer.load(...)` ahead of the sync `render` call (mirrors the existing native-vs-wasm async fragmentation around texture creation).
- [x] **Hidden docs** for `Renderer.load_js` / `load_py` / `load_mobile` (matches the established `render_*` / `create_texture_*` pattern; the per-platform shim signatures diverge enough from the canonical Rust API to deserve their own pages, even if those pages are only consumed via IDE hover).
- [x] **Tests:** `base_color_texture_with_raw_bytes_is_pending_until_render` (queues 1 entry, renders, asserts drained) and `texture_setter_with_existing_texture_skips_pending_queue` (eager path stays eager). Existing `pbr_samples_base_color_texture` and `pbr_with_no_user_textures_renders_with_defaults` continue to pass; the `From<&Texture>` route is preserved.
- [x] **Docs:** new `docs/api/core/renderer/load.md` (canonical), three hidden platform pages, plus updates to the five Material `*_texture.md` pages reflecting the new transport.
- [x] **Verification:** 246 lib tests + 149 doctests passing; `model_pbr_triangle` example still renders.

### Light: point + spot variants

Closes the placeholder "point and spot lights ship as follow-ups" item in the `Light` doc. The `Light` type grows two new constructors and the underlying uniform graduates from a single directional-only struct to glTF 2.0's `KHR_lights_punctual` shape: one binding with a `kind` discriminator plus the union of fields needed for all three variants. Sets up the glTF loader to faithfully reproduce a `KHR_lights_punctual` scene without lossy downgrades.

- [x] **`Light::point(position, color)`** at `src/scene/light.rs`. Radiates from a fixed world-space position with inverse-square distance falloff. Optional influence cutoff via `set_range(...)`; `range = 0` (the default) means unlimited.
- [x] **`Light::spot(position, direction, color)`**. Point light constrained to a cone aligned with `-direction`, with smooth roll-off between an inner and outer half-angle. Defaults: `inner = 0`, `outer = π/4`. Tighten via `set_cone_angles(inner_radians, outer_radians)`.
- [x] **Shared mutators across variants**: `set_position`, `set_intensity`, `set_range`, `set_cone_angles`, on top of the existing `set_direction` / `set_color`. Fields irrelevant to the active variant (e.g. `position` on a directional, cone angles on a point) are stored but ignored by the shader. The setters stay universal; the variant is fixed at construction.
- [x] **Shared getters**: `position()`, `intensity()`, `range()`, `inner_cone_angle()`, `outer_cone_angle()`. Existing `direction()` / `color()` unchanged.
- [x] **WGSL `Light` uniform** (replaces the old `DirectionalLight`). One binding at `@group(0) @binding(1)` holding `kind`, `intensity`, `range`, cone cosines, `position`, `direction`, `color`. `fs_main` branches on `kind`: directional uses `-direction` as the light vector with no falloff; point computes the surface→light vector from `position` and applies `1/d²` with optional `(1 - (d/range)⁴)` cutoff; spot adds a `smoothstep(outer_cos, inner_cos, cos_angle)` cone falloff on top of the point branch. `intensity` scales the final radiance uniformly so glTF intensity values feed in without Rust-side premultiplication.
- [x] **`Material::pbr` defaults** seed the new uniform fields (kind=directional, intensity=1, range=0, inner/outer cone defaults) so an unattached Material renders identically to the attached-directional case and a later `Light::point` / `Light::spot` attach overwrites a well-defined baseline.
- [x] **`Scene::ensure_render_defaults`** still injects a directional `Light::directional([0.3, -1.0, -0.4], [1, 1, 1])` when the user adds Models without a Light. No behavioural change to the hello-world path, just routed through the generalized binding.
- [x] **Cross-platform bindings** added for all 11 new public methods: `point`, `spot`, `position`, `intensity`, `range`, `inner_cone_angle`, `outer_cone_angle`, `set_position`, `set_intensity`, `set_range`, `set_cone_angles`, with full parity on Python (`snake_case`), JS / wasm-bindgen (`camelCase`), Swift / Kotlin via uniffi (`camelCase`). Setters return `Self` for builder chaining (Rust + Python); other bindings preserve the same shape with Arc reconstruction where the FFI layer requires it.
- [x] **Docs:** 11 new method pages under `docs/api/scene/light/` + `light.md` (module-level) rewritten with the variant table and a fields-vs-applies table. Every page carries a doctested example.
- [x] **Tests:** 6 new lib tests covering `point` / `spot` round-trip, range clamping, `point.attach` writes `kind=1` + position + intensity, `spot.attach` writes `kind=2` + cone cosines, `set_position` live propagation. 244 lib tests + 148 doctests passing.

### Scene: top-level container

Closes the v0.11.2 wishlist item "`Scene` object" and lands the abstraction the upcoming glTF loader builds on. `Scene` is the top-level real-world container; it owns one or more `Pass` entries underneath and implements `Renderable`, so the whole scene goes to the `Renderer` in a single call. The split mirrors glTF / USD: a Scene is a flat list of nodes (geometry, viewpoints, lights), and the renderer is the orchestrator that walks the scene to produce a frame.

- [x] **`Scene`** at `src/scene/scene.rs`. `Scene::new()` is sync (no `Renderer` argument, no async, nothing to await). The first `Scene::add` allocates a default `Pass` (`"Scene Default Pass"`) lazily; an empty Scene allocates no GPU bookkeeping at all. `Scene` is `Clone` (shallow Arc-share via an internal `SceneInner`) so cloned handles see each other's mutations.
- [x] **Unified `Scene::add<O: SceneObject>(&self, o: &O) -> Result<&Self, PassError>`** routes onto the lazy default Pass. Same trait surface as `Pass::add`: Models, Cameras, Lights, and any user-defined `SceneObject` go in through one method. Chainable with `?` (Models can fail at attach time on a Mesh-layout mismatch; Cameras and Lights always succeed). The TypeId of the added object is checked so the Scene knows whether the user has supplied a Camera / Light yet.
- [x] **`Scene::add_pass(&self, pass: &Pass) -> &Self`** appends a user-built Pass to a *pre-pass list*. Scene's `passes()` walks the extras in insertion order before the default Pass; that's the hook for shadow maps, depth pre-passes, and screen-space backdrops. For *post-effects* (bloom, tonemap, …), keep the post-pass outside the Scene and chain with another `renderer.render(...)` call.
- [x] **Default Camera / Light at render time.** When the user has added Models but never added a Camera or Light, `Scene::passes()` injects sensible defaults into the default Pass before returning so the "hello world" path renders something recognisable: `Camera::perspective(60°, 1.0, 0.1, 100.0)` looking from `[0, 0, 5]` at the origin with `+Y` up, and `Light::directional([0.3, -1.0, -0.4], [1, 1, 1])`. The injection is sticky (it fires once on the first `passes()` call, not on every render), and as soon as the user adds their own Camera / Light the defaults stop firing.
- [x] **`Renderable for Scene`.** `passes()` walks `extra_passes` first then the default Pass. `roots()` returns the same set of root pass nodes for dependency-DAG purposes. `renderer.render(&scene, &target)` works against any `Target` (window, texture, canvas).
- [x] **Cross-language bindings.** `Scene::new`, `add_model`, `add_camera`, `add_light`, `add_pass` exposed on Python (`add_model`, …), JS / wasm-bindgen (`addModel`, …), Swift / Kotlin via uniffi (`addModel`, …). The mobile `RenderableHandle` enum gains a `Scene(Arc<Scene>)` variant so `renderer.render(scene_handle, target_handle)` works on Swift and Kotlin. The web `Renderer.render` JS dispatch accepts `Scene` alongside `Pass` / `Shader` / `Mesh`. The Python `PyRenderable::from_any` recognises the `Scene` `renderable_type`.
- [x] **Docs:** `docs/api/scene/scene/{scene,new,add,add_pass}.md`; `docs/api/scene/_index.md` updated with `Scene` first.
- [x] **Example migration:** `examples/rust/examples/model_pbr_triangle.rs` switched from `Pass::add(...)` chain to `Scene::add(...)?` and `renderer.render(&scene, &target)`. Demonstrates the canonical top-level shape.
- [x] **Tests:** 9 new lib tests in `src/scene/scene.rs` cover empty Scene, lazy Pass creation, extras-then-default ordering, default-Camera + default-Light injection paths, both user-Camera and user-Light skip-default paths, shallow-Clone share semantics, and an end-to-end render through `Renderer::render(&scene, &target)`. 238 lib tests + 136 doctests passing.
- [x] **Pass keeps `add(SceneObject)`** as a lower-level escape hatch so existing tests / examples that drive a `Pass` directly still compile. Scene wraps it; Pass exposes it. The cross-platform typed shims (`add_model`, `add_camera`, `add_light`) live on both Pass *and* Scene because uniffi / pyo3 / wasm-bindgen can't dispatch on trait bounds, but every shim routes through the same Rust generic underneath.

### Camera + Light first-class types + unified `Pass::add(SceneObject)`

Closes the v0.11.2 follow-up "Camera object is planned" from the `Material::pbr` doc. Two new top-level types under `src/scene/`, plus a `SceneObject` trait that `Model`, `Camera`, and `Light` all implement so the Pass absorbs them through a single generic `add` method. The mental model lines up with glTF / USD: a scene is a flat list of nodes (geometry, viewpoints, lights), each attached to its parent.

- [x] **`Camera`** at `src/scene/camera.rs`. Constructors `Camera::perspective(fovy_radians, aspect, near, far)` and `Camera::orthographic(left, right, bottom, top, near, far)` (both built on glam's `*_rh` builders, matching wgpu's `[0, 1]` NDC depth range). Chainable `look_at(eye, target, up)` sets the view matrix and caches the world-space eye position. Accessors: `view_proj() -> [[f32; 4]; 4]` (column-major `proj * view`), `position() -> [f32; 3]`. State lives behind an `Arc<CameraObject>`, so a single Camera absorbed by many Passes propagates its `look_at` updates to all of them with no second `add` call.
- [x] **`Light`** at `src/scene/light.rs`. `Light::directional(direction, color)` is the directional constructor; accessors `direction()` and `color()`; setters `set_direction(...)` and `set_color(...)` mutate the Arc-shared state and propagate to every shader the Light has been wired into. Directional only for MVP; the type name reserves the abstraction for point / spot follow-ups.
- [x] **`SceneObject` trait** at `src/scene/object.rs`. `Model`, `Camera`, and `Light` all implement it; `attach(&self, pass)` is the entry point each value uses to plug itself into a Pass. Camera and Light also override `apply_to_shader(&self, shader)` for the live-propagation hook the pass replays whenever a new shader joins. Custom node types (anything that wants to inject a fixed set of uniforms or a draw entry) implement the trait the same way. The propagation list inside each Camera / Light compacts on every update by dropping `Weak<ShaderObject>` entries that no longer upgrade, so shaders that go out of scope don't leak through their attached components.
- [x] **`Pass::add<O: SceneObject>(&self, o: &O) -> Result<&Self, PassError>`** is the unified injection point. Chainable with `?` between calls: Models can fail at attach time when the Mesh layout doesn't match the Material's shader; Cameras and Lights always succeed. The older typed `Pass::add_model` is gone; same surface area, one method.
- [x] **Cross-language bindings** keep typed entries (`add_model`, `add_camera`, `add_light`) per-language because uniffi / pyo3 / wasm-bindgen can't dispatch on trait bounds, but all three route through the same Rust `Pass::add` internally. `Light` exposes `set_direction` / `set_color` for live mutation. Camera's `look_at` is callable repeatedly across all four languages; the Arc-shared backing means the second call propagates to every absorbing shader.
- [x] **Material no longer needs a Renderer.** `Material::pbr() -> Result<Self, ShaderError>` is sync; the five glTF texture slots start unbound, and the renderer fills them with neutral 1×1 fallbacks at draw time when the caller hasn't supplied a texture. `Renderer::default_pbr_textures` (the old async lazy-init) is replaced by `RenderContext::pbr_defaults` (sync, lazy-init, raw `wgpu::TextureView + Sampler`) and a small fallback branch in the bind-group setup. Removes the only public API that asked for a `&Renderer` at construction time and brings Material into line with the rest of the surface (Renderer is the orchestrator, not a constructor argument).
- [x] **Docs:** new `docs/api/scene/{camera,light}/` group + `docs/api/core/pass/add.md` (replaces the old per-method `add_model.md`). `docs/api/scene/_index.md` extended with Camera and Light after Model and Material. `docs/api/scene/material/pbr.md` and `docs/api/scene/material/shader.md` point at `pass.add(&camera)` / `pass.add(&light)` for camera + light state.
- [x] **Tests:** Camera covers projection sanity (perspective produces a non-trivial mat4, orthographic preserves `[3][3] = 1`), `look_at` view change + cached position, `pass.add(&camera)` round-trip through the PBR uniform surface, **a live-propagation test** (mutating the Camera after `add` updates every shader on the pass without a second `add`), and an order-agnostic test confirming a Camera added before any Models still wires through when Models join later. Light mirrors the same shape with `set_direction` / `set_color` propagation. 229 lib tests + 132 doctests, all passing.

### Mesh indices

- [x] **User-supplied indices via `Mesh::set_indices` / `Mesh::clear_indices`.** The auto path still dedupes vertices by full-attribute equality before producing an index array; fine for hand-built meshes, wrong for assets that already carry their own indexing (glTF loaders, OBJ importers, sharp-edge corners with split UVs / normals / tangents where two corners share a position but differ on other attributes). After `mesh.set_indices([...])` the mesh skips the dedup HashMap, packs every vertex in insertion order, and uses the indices the caller provided verbatim; `mesh.clear_indices()` returns to the auto-derived path. Plumbed through `MeshObject::indices_overridden` and a branch in `ensure_packed`; full parity across the four bindings (`set_indices` / `setIndices` on Python / JS / Swift / Kotlin).

### PBR texture sampling

`Material::pbr` graduates from factors-only to full glTF 2.0 PBR-MR texture sampling in the built-in default shader. The five canonical glTF map slots (`base_color_map`, `metallic_roughness_map`, `normal_map`, `occlusion_map`, `emissive_map`) are now declared at `@group(2) bindings 0..5` and combined with their matching factors in `fs_main`. Bind-group completeness is enforced at construction time via renderer-owned 1×1 fallback textures, so a Material that never had a texture set still renders correctly.

- [x] **Five textures sampled in `pbr_main.wgsl`.** Each combined with its factor per spec:
  - `albedo = material.base_color.rgb * textureSample(base_color_map, ...).rgb`
  - `metallic = material.metallic * sampled.bgr.r`; `roughness = material.roughness * sampled.bgr.g` (glTF channel layout)
  - `occlusion = mix(1.0, sampled.r, material.occlusion_strength)` multiplied into the diffuse term
  - `emissive = material.emissive * sampled.rgb` added on top
  - `normal_map` decoded into `[-1, 1]` and XY-scaled by `material.normal_scale`, applied as a placeholder additive perturbation to the world normal (full tangent-space TBN transform is a follow-up).
- [x] **Default-texture infrastructure on `Renderer`.** New `RwLock<Option<Arc<DefaultPbrTextures>>>` cell + `Renderer::default_pbr_textures()` (async, `pub(crate)`). Five 1×1 fallbacks chosen so `factor * sampled = factor` for the inert case: white base/occlusion/emissive, neutral tangent-space normal `(128,128,255)`, and `(R=0, G=1, B=1)` for metallic-roughness (so the `.bgr` swizzle in the shader passes both factors through unchanged). Lazy-init on first read; shared across every `Material::pbr` on the same renderer.
- [x] **Signature change: `Material::pbr() -> Result<Self, ShaderError>` → `Material::pbr(&Renderer) -> Result<Self, RendererError>` (async).** The Renderer is the natural place to source the default textures, and an explicit dependency at construction is cleaner than a renderer-aware draw loop. `RendererError` subsumes the old `ShaderError` (`#[from]`) and the new texture-creation error paths.
- [x] **Cross-platform bindings updated.** `pbr_py` blocks via `pollster::block_on` (Python surface is sync); `pbr_js` and `pbr_mobile` become async and take the renderer (`&Renderer` / `Arc<Renderer>`). No new free functions; every entry stays on `Material`.
- [x] **Tests added.** `pbr_seeds_default_texture_bindings` checks the five slots hold `UniformData::Texture` immediately after construction; `pbr_with_no_user_textures_renders_with_defaults` renders a triangle through the bind-group-complete defaults; `pbr_samples_base_color_texture` binds a 2×2 RGB-W texture across a full-quad triangle and asserts red and blue pixels appear in the readback.
- [x] **Docs reflow.** All six material method pages (`pbr.md`, `material.md`, plus the five `*_texture.md` pages) drop the "factors-only shader doesn't sample" caveat and gain a renderer in their example snippets; the related Model + Pass doc pages mirror the same example shape.

### Higher-level Scene objects: Material + Model

Closes the v0.11.2 wishlist item "per-mesh transform" without polluting Mesh with shader state. Adds two new top-level types under `src/scene/` and `src/material/` plus a new `mesh/` and `material/` shader-registry category. The original framing was "Mesh::set_transform"; rejected because uniforms belong on the Shader, not on Mesh. See `docs/api/scene/material/material.md` for the design notes.

- [x] **`mesh/transform` and `material/pbr` shader registry snippets.** Self-contained helper functions (no uniform declarations, no bind groups) so they compose into raw user shaders alongside `lighting/cook_torrance` and friends. `mesh_transform_position` / `mesh_transform_direction` / `mesh_transform_normal` and `pbr_shade(n, l, v, base_color, metallic, roughness, light_color)`. Embedded via two new `shaders-mesh` and `shaders-material` Cargo features; included in `shaders-3d` and `shaders-all`.
- [x] **`Material`** at `src/material/`. PBR or arbitrary `Shader` paired with the glTF 2.0 PBR-MR field set. Builder-style setters: `base_color`, `metallic`, `roughness`, `normal_scale`, `occlusion_strength`, `emissive`, `alpha_cutoff`, plus the five glTF texture slots (`base_color_texture`, `metallic_roughness_texture`, `normal_texture`, `occlusion_texture`, `emissive_texture`). `Material::pbr()` ships FragmentColor's built-in physically-based shader (Cook-Torrance + GGX + Smith + Schlick, Lambert diffuse) pre-seeded with sensible defaults, built by composing the two new registry slugs with the assembled `pbr_main.wgsl` from `src/material/`. Returns `Result<Material, ShaderError>` so slim builds without the helper features fail loudly at construction, not at draw. `Material::custom(shader)` wraps any shader and the same setters work best-effort under the matching uniform paths. `Material` is `Clone` (shallow Arc-share); cloning gives another handle to the same underlying state.
- [x] **`Model`** at `src/scene/`. Bundles `Mesh` + `Material` + per-Model 4×4 transform. `Model::translate(offset)` pre-multiplies a world-space translation; `rotate(axis, radians)` and `scale(factor)` post-multiply in local space (so a rotated-and-scaled Model spins and grows around its own origin). `set_transform([[f32; 4]; 4])` / `transform() -> [[f32; 4]; 4]` are the wholesale getter/setter pair (Rust doesn't allow getter/setter overloads on the same name; Python/JS bindings collapse them to a `.transform` property in their next pass).
- [x] **Per-Model transform rides a Pass-owned per-instance buffer.** `Pass::add_model` records a live reference (`Arc<RwLock<Mat4>>`) to each Model's transform. At render time the renderer groups entries by (Shader, Mesh), snapshots the current transforms, and uploads them as a single instance buffer with N rows: one draw call for N Models that share a Mesh+Material. WGSL locations 3..6 carry the four `vec4<f32>` columns; the default PBR vertex shader reconstructs the matrix in `vs_main`. The Mesh's own instance buffer is left alone: callers using `Mesh::add_instance(...)` directly for crowd-style rendering still work, and the per-Pass override beats the mesh-owned path when both are present. This is the architecturally-correct path that hits the renderer's existing pipeline-cache: one shader hash → one pipeline → one bind-group setup → one draw per Mesh, even with hundreds of Models on the Pass.
- [x] **`MeshObject::declare_model_instance_schema`** (pub-crate). `Pass::add_model` calls it on first use so the pipeline's instance VertexBufferLayout includes the four `model_0..3` columns; the Mesh's `insts` stays empty, no dummy data. `Shader::validate_mesh` was extended with a schema-only fallback (when `insts.is_empty()` but `instance_schema` is set) so the Pass-driven path passes layout validation.
- [x] **`Pass::add_model(&model)`** dedupes both the shader-attach and the mesh-attach by `Arc::ptr_eq`: many Models with the same Material → one shader entry on the pass; the same Mesh referenced twice → one entry on the shader. The transforms accumulate on `PassObject::model_entries`, the renderer expands them into the per-draw instance buffer.
- [x] **`Shader::duplicate` removed.** The prior (interim) approach kept a per-Model duplicate of the Shader to give each Model its own `mesh.model` uniform slot; the per-instance attribute path makes that unnecessary, and deep-cloning Shaders defeated the renderer's pipeline batching. `ShaderObject::source: Arc<str>` (which only existed to back `duplicate`) is gone too.
- [x] **`Model::sync_transform` removed.** The interim approach mutated the Mesh's instance buffer on every `translate`/`rotate`/`scale`; the live-reference path stores transforms on `Arc<RwLock<Mat4>>` and lets the Pass snapshot at draw time, picking up updates between `add_model` and `render` for free. `Model` is now pure Rust state with no GPU side effects.
- [x] **Cross-language bindings** for `Material`, `Model`, and `Pass::add_model` on Python / wasm-bindgen / uniffi (Swift, Kotlin). Builder-style chaining is Rust-only; in other languages setters mutate in place (Material is Arc-shared internally, so multi-statement setup gives the same shape without forcing a deep clone on every call).
- [x] **Docs:** new `docs/api/scene/{material,model}/` group (16 + 9 method pages, both `_index.md` group orderings), plus `docs/api/core/pass/add_model.md`.
- [x] **Example:** `examples/rust/examples/model_pbr_triangle.rs` renders a single PBR-shaded triangle through Model + Material + Pass::add_model, including camera and light overrides on the underlying Shader.
- [x] **Tests:** unit tests covering Material defaults, builder setters, shallow-clone share semantics, custom-shader silent no-op, Model transform composition (identity, pre-mult translate, post-mult rotate/scale, zero-axis rejection), live-reference share across `Model::clone`, Pass-entry dedupe for shared shaders, and the live-transform pickup between `add_model` and render. 213 lib tests, 116 doctests, all passing at this stage; the integrated v0.11.2 totals (after the parallel commits) land higher.
- [ ] **`Scene` object.** Collection of Models with traversal / sort / light management. Currently `src/scene/` houses `Model`, `Camera`, and `Light`; the module name reserves the spot for the collection type.
- [ ] **glTF loader.** Coming in a separate commit this cycle. The Material field set, indexed Mesh, AlphaMode/double-sided state, and PBR texture sampling are all in place ahead of the loader.

### Material alpha mode + double-sided

Wires the glTF 2.0 `alphaMode` and `doubleSided` flags through Material into the
renderer's pipeline state, closing the deferred item from the Material MVP. Pipeline
state is baked into the wgpu pipeline at build time, so different settings against
the same shader cache to distinct pipelines.

- [x] **New `AlphaMode` enum** at `src/material/alpha_mode.rs`: `Opaque` (default;
  depth-test on, blending off), `Mask` (fragment `discard`ed when
  `material.base_color.a < material.alpha_cutoff`), `Blend` (depth-test on,
  depth-write off, standard `SrcAlpha / OneMinusSrcAlpha` over-blend). Bound on
  every binding: `wasm_bindgen`-derived for JS, `pyclass(eq, eq_int)` for Python,
  `uniffi::Enum` for Swift / Kotlin.
- [x] **`Material::alpha_mode(self, mode: AlphaMode) -> Self`** and
  **`Material::double_sided(self, value: bool) -> Self`** builder setters on
  `src/material/mod.rs`. Cross-language shims (`alpha_mode` / `double_sided` in
  Python, `alphaMode` / `doubleSided` in JS / Swift / Kotlin) mutate in place
  through `&self` since uniffi / pyo3 / wasm-bindgen can't take `self` by value.
  Material stores both as `Arc<RwLock<…>>` so shallow `Clone` continues to share
  state across handles, matching the existing semantics.
- [x] **`ShaderObject` back-references** (`alpha_mode: RwLock<AlphaMode>` +
  `double_sided: RwLock<bool>`) carry the values from Material down to the
  renderer, which iterates `pass.shaders` at draw time and doesn't otherwise know
  which Material a shader belongs to. Material's setters write through to the
  shader's back-reference so the renderer reads consistent state every frame.
- [x] **`RenderPipelineKey` extended** with `alpha_mode` + `double_sided`. The
  renderer caches a separate pipeline per `(shader_hash, color_format,
  depth_format, sample_count, alpha_mode, double_sided)` tuple. `create_render_pipeline`
  picks `cull_mode: if double_sided { None } else { Some(wgpu::Face::Back) }`;
  the color target's `blend` field switches on `AlphaMode` (`Opaque` / `Mask` → no
  blending, `Blend` → `wgpu::BlendState::ALPHA_BLENDING`); `depth_stencil`'s
  `depth_write_enabled` flips off for `Blend`.
- [x] **`pbr_main.wgsl`** gained `alpha_mode_flag: u32` on the `PbrMaterial` uniform
  and a `material.alpha_mode_flag == 1u && material.base_color.a < material.alpha_cutoff`
  → `discard` branch at the top of `fs_main`. `Opaque` and `Blend` ignore the flag;
  their semantics live in pipeline state, not fragment-shader logic.
- [x] **Tests:** five new ones in `src/material/mod.rs::tests`:
  `alpha_mode_setter_updates_shader_back_reference`,
  `double_sided_setter_updates_shader_back_reference` (state propagation),
  `mask_mode_discards_transparent_fragments`,
  `opaque_mode_keeps_below_cutoff_fragments` (Mask discard end-to-end through
  the renderer with an offscreen `TextureTarget` readback), and
  `double_sided_true_renders_back_facing_triangle` (back-face cull flip).
  218 lib tests + 118 doctests passing.
- [x] **Behaviour notes for raw `Shader` callers.**
  - The renderer's previous default of `wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING`
    on every pipeline is gone; blend is now driven entirely by Material's
    `alpha_mode`. Shaders that hit the renderer without a Material in the
    middle render with blending **off** (the new `AlphaMode::Opaque` default).
    Callers relying on the old behaviour migrate by wrapping the shader in
    `Material::custom(shader).alpha_mode(AlphaMode::Blend)`.
  - To keep no-Material rendering working for back-facing geometry, the
    `ShaderObject` default for `double_sided` is `true` (which sets
    `cull_mode: None`). `Material::pbr` and `Material::custom` both flip it
    to `false` on the back-reference so the Material path follows glTF 2.0's
    single-sided default.
- [x] **Docs:** `docs/api/scene/material/alpha_mode.md` and
  `docs/api/scene/material/double_sided.md` lead with what the flag does, when
  to reach for each variant, and the glTF 2.0 mapping. `docs/api/scene/material/material.md`
  and `alpha_cutoff.md` updated to drop the "not yet wired" caveat.

### `R16Unorm` and the 16-bit norm family

Diagnosed against a consumer's failing shader path: an `R16Unorm` `TextureMipChain` that round-tripped fine through `prepare → from_chain → device.create_texture` produced a silently-invalid texture on Apple Silicon, then exploded on first `create_view()` with an `InvalidResource` cascade that drowned stderr 60 times per second. Same for `Rg16Unorm`, `Rgba16Unorm`, and the three `*Snorm` variants.

- [x] **Adapter feature probe widened.** `request_device` opportunistically requests `TEXTURE_FORMAT_16BIT_NORM` and `FLOAT32_FILTERABLE` alongside the texture-compression features. Apple Silicon and modern desktop adapters get a working `R16Unorm` + `TEXTURE_BINDING` path; opt-in via `adapter.features().contains(...)` so non-supporting adapters still get a working device.
- [x] **Fail-fast on adapters without the feature.** New `TextureError::UnsupportedFormatForUsage { format, missing_feature }` variant + `check_format(features, format, usage)` guard at every user-controlled `device.create_texture` site (`TextureObject::{new, from_input}`, KTX2 loader). Typed error at the API boundary instead of the cascade-50-frames-later landmine.
- [x] **wgpu validation scope around bind-group + view creation.** `RenderContext::validate(label, op)` folds the prior `create_bind_group_checked` into a generic that wraps any wgpu call whose validation failure would otherwise leak via `on_uncaptured_error`. `process_render_pass` and `process_compute_pass` wrap both `tex.create_view()` and `device.create_bind_group()`. Single `RendererError::ValidationError { label, message }` instead of the 4-tier validation cascade.
- [x] **Regression test:** `renderer::tests::render_with_r16unorm_texture_smoke` exercises the full consumer path (R16Unorm prepared chain → bound via uniform → `renderer.render` → asserts no error). Plus pure-table units (`format_feature_covers_16bit_norms_only`, `check_format_fails_fast_when_feature_absent`).

### Method naming pass: single canonical name per operation

Audit + cleanup across the public API and internal helpers. Rule: 1 verb or max 3 words; suffixes only when they disambiguate genuinely distinct inputs (`from_file` vs `from_bytes`). Platform binding suffixes (`_js` / `_mobile` / `_py` / `_android` / `_ios`) are forced by uniffi/wasm-bindgen/pyo3 needing distinct signatures and stay.

- [x] `Pass::add_mesh_to_shader(mesh, shader)` removed (was `shader.add_mesh(mesh)?` ignoring `&self`). Callers use `shader.add_mesh(mesh)` directly. Per-platform wrappers, doc page, example files removed.
- [x] `PassObject::set_color_target_id(id)` → `set_color_target(id)`; same for `set_depth_target_id` → `set_depth_target`. The arg name carries the type.
- [x] `App::on_event_kind(kind, f)` → `on_event(kind, f)`; same for `on_window_event_kind` and `on_device_event_kind`. Catch-all `on_event(f)` / `on_device_event(f)` (no `kind` arg) variants removed; kind-filtered registration is the only way.
- [x] Free fns `create_external_texture_from_native(_r, _ptr)` and `create_external_texture(_r, _video)` → `ExternalTextureHandle::from_native(renderer, ptr)` and `ExternalTextureHandle::from_video(renderer, video)`. Implementation still a stub; the API moves to where it belongs.
- [x] `Target` trait gained async `get_image() -> Vec<u8>`, mirroring `Texture::get_image()`. `TextureTarget::get_image_async` removed (the trait method covers it). `WindowTarget::get_image` is a stub returning `Vec::new()`; proper screen capture from a presentable surface needs `COPY_SRC` on the swapchain config (queued).
- [x] `TextureObject` constructor family folded 5 → 1: `from_file` / `from_bytes` / `from_raw_bytes` / `from_image` / `from_chain` (latter two renamed from `from_loaded_image` / `from_prepared_chain`) collapsed into `TextureObject::from_input(context, input)`. `Renderer::create_texture` shrank from ~165 lines (8-arm match + duplicated registration) to ~22.
- [x] Sync/async pair unification using the `blocking` submodule convention from `reqwest::blocking`. `shader/input.rs` `resolve_async` → `resolve` (async); the prior sync `resolve` → `blocking::resolve`. Same for `resolve_part` and `fetch_url`. `texture/read.rs` `read_texture_object_async` → `read_pixels` (async); sync read path gone (only consumer was `Target::get_image`'s removed sync variant).
- [x] Internal helper renames (renderer + texture + mesh): `try_with_validation` → `validate`; `configure_surface_with_context` → `configure_surface`; `try_get_frame_with_retry` → `acquire_frame`; `create_vertex_buffer_layouts` → `vertex_buffer_layouts`; `create_bind_group_layouts` → `bind_group_layouts`; `available_compression_features` → `format_features`; `format_supports_cpu_mipmaps` → `supports_cpu_mipmaps`; `build_mip_chain_bytes` → `build_mip_chain`; `write_raw_bytes_levels` → `write_levels`; `wrap_raw_bytes_as_dynamic_image` → `bytes_as_image`; `infer_format_from_image` → `infer_format`; `validate_format_for_binding` → `check_format` + `required_feature_for_binding` → `format_feature`; `first_vertex_location_map` → `vertex_location_map` + `first_instance_location_map` → `instance_location_map`; `create_gpu_vertex_buffers` → `upload_vertex_buffers`.
- [x] `Pass::from_shader_object` + `add_shader_object` (private internal duplicates of the public `Pass::from_shader` / `add_shader`) folded. `PassObject` versions take `Arc<ShaderObject>` directly, public `Pass` wrappers do the `&Shader → Arc` extraction at the boundary.

### API thinning: single-method-per-operation, single transport

Multi-slice refactor: collapses `_with_*` method families into single canonical methods with `From<T>` impls; unifies the cross-language surface so JS/Python/Swift/Kotlin see the same shapes; merges three texture-input transports (`TextureSpec` / `StorageTextureInput` / `PrepareSpec`) into one shared `TextureInput`.

- [x] **Naming.** `TextureData` is the source enum (`Empty | Bytes | Path | Url | DynamicImage | Ktx2* | CloneOf | Prepared`); `TextureInput { data, options }` is the universal transport; `TextureOptions` carries `size: Option<Size>`, `format`, `sampler`, `mipmaps`, `usage: Option<u32>` (raw bit mask, with a `with_usage(wgpu::TextureUsages)` builder for typed Rust call sites).
- [x] **`Renderer::create_texture(input)` is the single texture-creation entry.** Drops `_with_size`, `_with_format`, `_with`, and `_prepared`. JS/Python collapse to one method with optional `options` arg; mobile takes uniffi-marshallable `TextureInputMobile` enum + `TextureOptions` (`uniffi::Record`); Swift/Kotlin extension files supply natural overloads.
- [x] **`Renderer::create_storage_texture(input)` is the single storage entry.** Drops `_with_data` and the separate `StorageTextureInput`. `From<(size, format)>` produces empty form; `From<(size, format, bytes)>` produces seeded. `options.usage` overrides the default storage-usage mask.
- [x] **`TextureMipChain::prepare(input)` is the single CPU-prep entry.** Drops `PrepareSpec`. Tuple `From` impls cover `(bytes, format)` for encoded and `(bytes, format, size)` for raw. `prepare` validates `data` is a sync-friendly variant (`Bytes`, `DynamicImage`, `Path`) and surfaces a typed `InvalidInput` error pointing at the right entry point for variants it can't handle (`Url` → fetch first, `Ktx2*` → already pre-baked, `Prepared` → already a chain, `Empty` → nothing to prepare).
- [x] **`Renderer::render(renderable, target)` is the single render entry on every platform.** Mobile uniffi binding's split `renderShader` + `renderShaderToTexture` replaced by `RenderableHandle` (`Shader | Pass | Mesh | Passes`) + `TargetHandle` (`Window | Texture`) `uniffi::Enum`s. Swift/Kotlin extensions supply natural overloads. `Pass` and `Mesh` derive `uniffi::Object` so they can ride inside the handle enums.
- [x] **Cross-language brand detection for `TextureMipChain` handles in JS** via the existing `__fc_kind` + `__wbg_ptr` anchor pattern (`impl_js_bridge!`).
- [x] **Net surface delta:** ~9 Rust methods → 4. ~24 FFI shims → ~9. Three transport types → one. Same `TextureInput` flows through all three texture paths.
- [x] **Trade-offs accepted:** "size required for storage" + "data must be sync-friendly for prepare" are runtime validations, not compile-time guarantees. Same convention as the existing KTX2 paths silently ignoring `options.format` / `options.mipmaps`.
- [ ] **Follow-up (not in this change):** structurally splitting `src/renderer/platform/mobile/` into per-language `ios.rs` + `android.rs` so each language's idioms get their own translation layer.

### Texture creation off the main thread

- [x] **`Renderer::create_texture` no longer blocks the calling thread on CPU work.** Decoding (`image::load_from_memory` / `image::open`), the `image::imageops::resize` Triangle-filter mipmap chain, and the per-level `wgpu::Queue::write_texture` calls run on a single named worker (`fragmentcolor-bg`) on every native target. Affects `Bytes` / `Path` / `Url` (post-fetch) / `DynamicImage`; KTX2 inputs stay inline (cheap to decode). Worker is process-wide and lazy.
- [x] **Wasm keeps today's behavior.** `wgpu::Device` and `wgpu::Queue` are `!Send` on `wasm32`; the `cfg(wasm)` path runs prep inline. No regression vs. previous releases; web users who need real parallelism can move decode + prep into a Web Worker themselves.
- [x] **`TextureMipChain` exposed on every binding.** Two constructors: `TextureMipChain::prepare(bytes, format)` (encoded; decodes via `image`) and `TextureMipChain::prepare_raw(bytes, size, format)` (raw pixel bytes). Consumed via `Renderer::create_texture_prepared(chain)` (cross-language) or `Renderer::create_texture(TextureInput::Prepared(chain))` (Rust ergonomics). `Clone` via internal `Arc<Vec<Vec<u8>>>` so handing the same chain to multiple textures doesn't duplicate the byte buffers.
- [x] **Cross-language exposure:** bound via `#[wasm_bindgen]` (Web), `#[pyclass]` + `#[staticmethod]` (Python), and `#[uniffi::constructor]` (Swift/Kotlin). Accessors (`format()` / `baseSize()` / `levelCount()` / `level(i)`) let callers inspect or persist a chain. `TextureFormat` derives `uniffi::Enum`, `Size` derives `uniffi::Record`.
- [x] **Typed error surface for prepare:** `MalformedImageError(image::ImageError)` (decode failure), `UnsupportedMipmapFormat { format }` (target format unsupported by CPU mipmap dispatcher), `InvalidInput(String)` (bytes parsed but didn't match declared shape: zero size, byte count too small for `bpp * width * height`).
- [x] **`prepare_raw` accepts `impl Into<Size>`** on the canonical Rust signature; bindings still take a concrete `Size` (uniffi / wasm-bindgen / pyo3 don't marshal generics).
- [x] **No new dependencies.** Worker uses `std::thread` + `std::sync::mpsc` for the job queue and `futures::channel::oneshot` (already a dep) for the per-call reply.
- [ ] **Out of scope (deferred):** multi-worker pool, drop-cancellation, shader-compile / buffer-upload offload, `TextureInput` marshalling across FFI for the prepared-chain path.

### KTX2 container support (BC / ETC2 / ASTC + uncompressed)

- [x] **`TextureInput` gained three KTX2 variants** (`Ktx2Bytes(Vec<u8>)`, `Ktx2Path(PathBuf)`, `Ktx2Url(String)`) through the same `Renderer::create_texture(_with)` entry points as JPEG/PNG. Pure-Rust parsing via the `ktx2` crate; no C++ build pollution.
- [x] **The KTX2 path trusts the file's declared format and pre-baked mip chain.** `options.format` and `options.mipmaps` are intentionally ignored for KTX2 inputs; encoders pick the format and chain on purpose, and doing it twice would only round-trip through a worse approximation.
- [x] **Compression GPU features requested opportunistically at device creation:** `TEXTURE_COMPRESSION_BC` / `_ETC2` / `_ASTC` (and SLICED_3D / HDR variants) per adapter advertisement. Adapters without a given feature still get a working device; KTX2 loads of unsupported formats fail at upload with a clear error rather than crashing inside wgpu validation.
- [x] **Format coverage** (Vulkan `VkFormat` → `wgpu::TextureFormat`): RGBA8 UNORM/SRGB, BGRA8 UNORM/SRGB, R8/Rg8/R16/Rg16/Rgba16 UNORM, RGBA16F, BC1-BC7 (UNORM and SRGB), ETC2 RGB/RGBA/RGB-A1 (UNORM and SRGB), ASTC 4×4 and 8×8 (UNORM and SRGB). Other VkFormats fail loudly.
- [ ] **Out of scope (deferred):** Basis Universal transcoding (`VK_FORMAT_UNDEFINED` payloads), supercompression schemes (zstd / zlib / BasisLZ), cube maps, array textures, 3D textures, progressive intra-file mip streaming.

### Wider source-image format support (R8 / Rg8 / R16 / Rg16 / Rgba16)

- [x] **`Renderer::create_texture` decodes images into the right pixel buffer for the target format**, instead of `to_rgba8` for everything. 16-bit grayscale PNG with `format: TextureFormat::R16Unorm` → `to_luma16` (no upper-byte truncation). Same dispatch handles `R8Unorm` (`to_luma8`), `Rg8Unorm` (`to_luma_alpha8`), `Rg16Unorm` (`to_luma_alpha16`), `Rgba16Unorm` (`to_rgba16`). Mipmap generation runs over the typed `ImageBuffer`, preserving precision at every level.
- [x] **`TextureFormat` gained `R16Unorm` and `Rg16Unorm` variants** on every binding. JS bridge enum numeric ordering shifted to insert the new variants; JS callers passing format integers directly should re-read from regenerated bindings.
- [x] **`from_raw_bytes` mipmap support generalized** to the same set of formats. 16-bit byte slices decoded to `Vec<u16>` via `from_le_bytes` before resampling: alignment-safe, matches WebGPU's little-endian element order.
- [x] **Pre-existing bug fixed as a side effect:** a 16-bit PNG fed through `create_texture(path)` was created with format `R16Unorm` (per `image::ColorType` inference) but written with `to_rgba8` bytes (4 bpp into a 2-bpp texture), producing garbled rows. The new dispatch makes the inferred format and byte layout agree.

### Source-image mipmaps + trilinear filtering

- [x] **`Renderer::create_texture` and friends now generate a full mipmap chain at upload** for source images (file path, encoded bytes, URL, `DynamicImage`). Combined with the default linear sampler picking `mipmap_filter: Linear` when `smooth: true`, textured surfaces get proper trilinear filtering at any zoom or rotation. Fixes the moving-moiré artifact when zooming out on a textured quad whose source image has high-frequency detail (canvas weave in painted JPEGs being the canonical case). CPU-side via `image::imageops::resize` with the Triangle filter; resampling runs directly on the source bytes (sRGB-encoded for color content). No GPU work added at render time.
- [x] **`TextureOptions.mipmaps: bool` (default `true`).** Set `false` to skip CPU work for textures that won't be sampled at distance (single-pixel sentinels, render targets sampled 1:1).
- [x] **`TextureOptions.format` honored on every input arm** of `create_texture_with`, not just the raw-bytes-with-size path. Sentinel default `TextureFormat::Rgba` still means "infer from input"; explicit variants override. Unblocks loading RGBA8 PNGs as `Rgba8Unorm` (linear bytes) for normal-map / non-color data without the `from_raw_bytes` round-trip.
- [x] **`TextureObject` constructors generalized in place.** `from_file` / `from_bytes` / `from_raw_bytes` / `from_loaded_image` each gained `(format_override, generate_mipmaps)` trailing parameters (`pub(crate)`).
- [x] Mipmap generation runs only for color formats sharing the source's RGBA8 byte layout (Rgba8 family + Bgra8); other formats stay single-level even when `mipmaps: true`.

### Shader composition

- [x] **`Shader::new` accepts arrays.** Signature is `Shader::new(impl Into<ShaderInput>)`, with `From` impls classifying a single string into raw WGSL/GLSL **source**, registry **slug** (`"sdf2d/circle"`), `https://` **URL**, or local **path**. Pass an array of any combination: parts resolved (fetched / read / looked up), deduped by source hash, concatenated in order before naga validation. No wrapping or auto-injection; invalid input fails loudly.
- [x] Equivalent forms: `Shader::new(src)`, `Shader::new("sdf2d/circle")`, `Shader::new(["sdf2d/circle", "noise/simplex2", main_src])`. Existing call sites passing `&str` / `String` / `&String` keep working unchanged.
- [x] **`Shader::set_registry(base_url)`** overrides the slug base URL (default `https://fragmentcolor.org/shaders/`). Process-wide; tests use a thread-local override stack.
- [x] **GLSL** is supported only as a single part; mixing GLSL with WGSL or with other parts is rejected.
- [x] **Behaviour change:** a string of shape `^[a-z][a-z0-9_]*/[a-z0-9_]+$` (single line, ≤128 chars) is now classified as a **slug** rather than parsed as raw WGSL. Existing `Shader::new("sdf2d/circle")` callers fetch from the registry instead of erroring.
- [x] **JS/WASM:** `new Shader(input)` accepts `string | string[]`. URL/slug parts must be resolved via `await Shader.fetch(input)` (also accepts `string | string[]`). New `Shader.setRegistry(baseUrl)` static.
- [x] **Python:** `Shader(input)` accepts `Union[str, list[str]]`. New static `Shader.set_registry(base_url)`.
- [x] **Swift / Kotlin (uniffi):** `Shader.new(source)` and new `Shader.compose(parts)` constructors, plus a free `set_shader_registry(base_url)` function. Extension shims provide a single overloaded `Shader(_:)`.

### Vertex attribute name constants

Canonical string keys for the common per-vertex channels so the (forthcoming) glTF loader, user shaders, and `Vertex::set` call sites all agree on attribute names without bikeshedding. They're plain `&'static str` literals: `vertex.set(Vertex::UV0, [...])` and `vertex.set("uv0", [...])` are equivalent.

- [x] `Vertex::POSITION = "position"` (implicit via `Vertex::new`)
- [x] `Vertex::NORMAL = "normal"`
- [x] `Vertex::TANGENT = "tangent"`
- [x] `Vertex::UV0 = "uv0"`, `Vertex::UV1 = "uv1"`
- [x] `Vertex::COLOR0 = "color0"`, `Vertex::COLOR1 = "color1"`
- [x] Test: `mesh::vertex::tests::attribute_name_constants_match_string_lookup` round-trips the constants through `set` and asserts the string values.

### Pass depth-test (documentation polish, no API change)

The depth-test path was already implicit: `Pass::add_depth_target(depth_tex)` enables depth-test and depth-write for the pass; not calling it means painter's-algorithm rendering. The behaviour is consistent and adequate for 3D mesh occlusion, but the docs underplayed it.

- [x] `docs/api/core/pass/add_depth_target.md` rewritten to lead with "depth-test is enabled" and surface the opt-out (just don't attach). Example reframed as a 3D-mesh-over-quad pattern matching the canonical consumer use case.
- [ ] (Deferred) Explicit `Pass::set_depth_test_enabled(bool)` / `set_depth_write_enabled(bool)` setters for the depth-attached-but-test-disabled case (translucent overlays). Holding until a real consumer needs it.

### Texture group restructure (Mipmap + Texture out of `core/`)

- [x] Rename `TextureMipChain` → `Mipmap` (Rust type, FFI brand strings, `__fc_kind`, all cross-platform bindings). Method renames: `prepare` → `build`, `base_size` → `size`, `level_count` → `count`. `format` and `levels` unchanged.
- [x] Restructure docs groups: new `docs/api/texture/` containing `Texture` and `Mipmap`. `core/` shrinks to `Renderer`, `Shader`, `Pass`. Texture is reframed as an external input alongside Vertex (in `geometry/`); future texture helpers (Sprite, Atlas, etc.) get a home.
- [x] Migrate platform examples: `platforms/{python,web,kotlin,swift}/examples/core/texture/` → `…/examples/texture/texture/`; `…/examples/core/texture_mip_chain/` → `…/examples/texture/mipmap/`.
- [x] Update `lsp_doc("…")` paths, `generated/api_objects.txt`, `generated/api_map.rs`, `_index.md` files for both old and new groups.

## 0.11.1 Embedded shader registry by default, network behind a feature flag

Patch release that unblocks the Linux PyPI publish path. v0.11.0 shipped to crates.io, npm, the Swift xcframework, and the Android AAR, but the PyPI wheels never landed. Reframes the underlying decision: a graphics library should not drag a TLS stack into every consumer's dep tree.

### Defaults

- **The whole public shader library (about 86 KB of WGSL across 233 files) is now embedded by default** on every native build. `Shader::new("sdf2d/circle")` resolves from the binary, no network needed. Slim down with `default-features = false` if size matters.
- **Registry URLs short-circuit to the embedded library.** A URL of the form `<registry-base>/<category>/<name>.wgsl` (default base `https://fragmentcolor.org/shaders/`) is detected as a registry URL and resolved locally on native, no network round-trip. So `Shader::new("https://fragmentcolor.org/shaders/sdf2d/circle.wgsl")` and `Shader::new("sdf2d/circle")` produce the same result, and the verbose URL form keeps working in docs and quickstart examples on every platform.
- **The web build (`build_web`) skips the embedded library** via `--no-default-features`. Slugs and registry URLs resolve over the browser's `fetch()` against the live registry instead, keeping the wasm bundle small.
- **`network` is a new Cargo feature, off by default on every native target.** When on, `ureq` is compiled with `native-tls` so off-registry URLs (`https://example.com/foo.wgsl`, etc.) and arbitrary `Renderer::create_texture(url)` calls fetch over the platform's system TLS stack (OpenSSL on Linux, Schannel on Windows, Secure Transport on macOS). When off, those non-registry URL paths return a typed `NetworkError::feature_disabled()` with a clear "rebuild with `--features network`" message.
- **No API drift.** `Shader::fetch` and `Renderer::create_texture` accept the same input shapes on every binding. Slugs, registry URLs, file paths, and raw source work everywhere; only off-registry URL fetches change behaviour based on how the binary was compiled.

### Internal

- New `crate::net::NetworkError` replaces direct `ureq::Error` references in `FragmentColorError::NetworkRequest`, `RendererError::NetworkRequestError`, and `ShaderError::RequestError`. `From<ureq::Error>` is provided when the feature is on so call sites keep using `?` unchanged.
- `ureq` is now an optional dep gated on the `network` feature. Without the feature, `ring`, `rustls`, and `openssl-sys` are out of the default dep tree entirely; PyPI wheels build cleanly across every Linux arch.

### CI

- Regenerate `.github/workflows/publish_py.yml` with maturin 1.13.1. Picks up newer action versions (`actions/checkout@v6`, `setup-python@v6`, `upload-artifact@v6`, `download-artifact@v7`, `attest-build-provenance@v3`), drops the Node.js 20 deprecation warnings, switches to `uv publish`, builds wheels on every PR (so cross-compile breakage is caught before release), and adds a Windows ARM64 target.

### Metadata

- Bump PyPI development status from `2 - Pre-Alpha` to `3 - Alpha`. Five language bindings ship; the API is settling but pre-1.0 churn continues.

### Migration

Most call sites need no changes. Registry URLs (`https://fragmentcolor.org/shaders/<category>/<name>.wgsl`) now resolve from the embedded library on native, with no network needed and no code changes required.

Off-registry URLs (anything that doesn't match `<registry-base>/<category>/<name>.wgsl`) on native:

- **If you can use a registry slug or registry URL,** that's the easiest path. It works offline on every platform.
- **If you need an arbitrary URL on native,** add `--features network` to your build (`cargo add fragmentcolor --features network`, or rebuild the Python wheel with `maturin build --features network`).
- **The web binding is unchanged.** `Shader.fetch(url)` keeps working through `fetch()` in the browser for any URL.

## 0.11.0 Swift & Kotlin with Uniffi

### API renames + parity closures

- **`Renderer::create_external_texture_from_html_video` → `Renderer::create_external_texture`.**
  The old name baked the only existing source type into the API; renaming makes the surface
  cross-platform (every binding now exposes `createExternalTexture` / `create_external_texture`
  with a per-language source argument: `HTMLVideoElement` on Web, `CVPixelBuffer` raw pointer
  on iOS via uniffi, `SurfaceTexture` raw pointer on Android via uniffi). The Rust core impl is
  still a stub returning `RendererError::Error("not implemented yet")` on every platform.
  The API surface is in place so callers can write portable code paths now, and the
  per-platform plumbing fills in over 0.12.6 (see ROADMAP). `ExternalTextureHandle` is no
  longer `#[cfg(wasm)]`-gated. It derives `uniffi::Object` on mobile and stays
  `#[wasm_bindgen]` on Web.
  - The public doc is **parked at
    `docs/api/core/renderer/hidden/create_external_texture.md`** until the implementation
    lands, so the website doesn't advertise an API that throws on every call. The
    `lsp_doc` reference on the Web wrapper now points at the hidden path; the four
    per-language hidden overrides (`_js.md` / `_py.md` / `_swift.md` / `_kotlin.md`)
    were renamed to match and stay parked alongside it as future-implementation
    references. Move them back to the public top-level once 0.12.6 ships.
  - Removed: `docs/api/core/renderer/create_external_texture_from_html_video.md` and the four
    matching hidden overrides; `platforms/{web,python,swift,kotlin}/examples/core/renderer/create_external_texture_from_html_video.*`
    (regenerated under the new name).
- **`Texture::set_sampler_options` is now uniffi-exported.** `SamplerOptions` (now
  `#[derive(uniffi::Record)]`) and `CompareFunction` (now `#[derive(uniffi::Enum)]`) are
  in the foreign import list for Swift / Kotlin. Added `Texture::set_sampler_options_mobile`
  shim in the new [src/texture/platform/mobile.rs](src/texture/platform/mobile.rs); the
  Swift / Kotlin doc examples now show the idiomatic
  `SamplerOptions(repeatX: true, ...)` constructor instead of a placeholder. Dropped
  `SamplerOptions` from the foreign-import filter in `scripts/convert.rs` so the import line
  survives transpilation. Closes one of the parity gaps surfaced by the new doc-example
  healthchecks.

### API removals

- **`Frame` is gone.** The type was a thin collector over `Pass` objects; after the
  Sep 30 DAG refactor it held no capability that `Pass::require()` (dependency graph) or
  an iterable of `Pass` (sequential rendering) didn't already cover. `Renderer::render`
  already accepts `&Pass`, `&Vec<Pass>`, `&[&Pass]`, `&[Pass]`, and `&Vec<&Pass>`, so every
  Frame use-case transliterates directly. Because every public symbol multiplies across
  5 language bindings (Rust, JS, Python, Swift, Kotlin), the API surface reduction is
  worth the transliteration cost, and 0.11.0 is not yet published, so no consumers exist.
  - Migration: replace `let mut frame = Frame::new(); frame.add_pass(&a); frame.add_pass(&b);
    renderer.render(&frame, &target);` with `renderer.render(&vec![a, b], &target);`.
  - Python: `frame = Frame(); frame.add_pass(p); renderer.render(frame, t)` →
    `renderer.render([p], t)`.
  - JS: `const frame = new Frame(); frame.addPass(p); renderer.render(frame, t)` →
    `renderer.render([p], t)`.
  - Removed: `Frame`, `FrameError`, `docs/api/core/frame/**`, and the per-language
    `platforms/*/examples/core/frame/**` stubs.

Initial Swift and Kotlin bindings via [uniffi](https://github.com/mozilla/uniffi-rs). Struct names match
the Rust core (`Renderer`, `Shader`, `Pass`, `Size`, `Region`, `WindowTarget`, `TextureTarget`,
`Texture`, `SamplerOptions`, `CompareFunction`) so the public API reads the same across every
supported platform.

### Build system

- Add `uniffi = "0.29"` under non-wasm targets and wire the build-script helpers for scaffolding.
- Add `uniffi-bindgen` binary and top-level `uniffi.toml` config.
- Enable `uniffi::setup_scaffolding!()` for all non-wasm builds.
- Add iOS dependencies: `objc2-foundation`, `objc2-quartz-core`.
- Add Android dependencies: `jni`, `jni_fn`, `ndk-sys`, `raw-window-handle`.

### API additions (iOS/Android only)

- `Renderer::create_target_ios(metal_layer_ptr: u64)` (iOS, exposed as
  `Renderer.createTarget(metalLayerPtr:)` / `Renderer.createTarget(layer:)` via the Swift
  extension): build a `WindowTarget` from an existing `CAMetalLayer` pointer.
- Raw `#[jni_fn]` Android entry point (`create_window_target_from_surface`) that returns
  `Arc<WindowTarget>` from an `android.view.Surface`; the Kotlin extension wraps it as
  `Renderer.createTarget(surface:)`. Cannot go through uniffi because uniffi does not marshal
  `JNIEnv*`.
- `Renderer::create_texture_target_mobile(width, height)`: uniffi-friendly concrete-typed
  variant of `create_texture_target`; Swift / Kotlin see it as `createTextureTarget`.
- `Renderer::render_shader_mobile` / `render_shader_texture_mobile`: uniffi variants of
  `render` (one per target type, since uniffi can't marshal `impl Trait`). Swift / Kotlin
  extensions recombine them into a single overloaded `render(shader, target)`.
- `Shader::new_mobile(source: String)`: uniffi constructor; Swift sees it as `Shader(source:)`
  via uniffi's `convenience init`, Kotlin sees it as `Shader.new(source)`.
- `Texture::set_sampler_options_mobile(opts: SamplerOptions)`: uniffi shim; Swift sees it as
  `setSamplerOptions(opts:)`, Kotlin as `setSamplerOptions(opts)`. Closes the parity gap
  with the existing Web (`setSamplerOptions`) and Python (`set_sampler_options`) wrappers.
- `Renderer::create_external_texture_mobile(source_ptr: u64)`: uniffi shim that takes a
  raw pointer to a native video-frame source (`CVPixelBuffer` on iOS, `SurfaceTexture` on
  Android). Stub today (returns `FragmentColorError::Render("not implemented yet")`) but
  the API surface exists on every binding.

### CI + release pipeline

- Add `.github/workflows/healthcheck_ios.yml` (macos-14 runner with Xcode): builds the
  xcframework and runs `./healthcheck ios` on an iPhone simulator on every PR that touches
  `src/**`, `platforms/swift/**`, mobile-relevant build inputs, or the `docs/api` folder.
- Add `.github/workflows/healthcheck_android.yml` (ubuntu-latest runner with KVM): builds
  `libfragmentcolor.so` for the emulator ABI via `cargo-ndk`, boots an Android emulator,
  and runs `./gradlew connectedAndroidTest` with the uniffi-generated Kotlin bindings.
- Add `.github/workflows/publish_swift.yml`: on release, builds the xcframework, zips it,
  attaches `fragmentcolor.xcframework.zip` to the GitHub Release as an asset, and records
  the SPM checksum.
- Add `.github/workflows/publish_android.yml`: on release, builds the `.so` for all 4 ABIs
  and uploads `fragmentcolor-<version>.aar` to the GitHub Release.
- Extend `.github/workflows/post_publish_update.yml`: after waiting for npm, PyPI **and**
  the xcframework release asset, pins the top-level `Package.swift` `fragmentcolorVersion`
  / `fragmentcolorChecksum` to match the just-published release and rolls that into the
  post-publish consumer-update PR.

### Doc-example healthchecks (Swift + Kotlin)

The build script already transpiled every `docs/api/**.md` Rust example into a Swift
and a Kotlin sibling under `platforms/swift/examples/` and `platforms/kotlin/examples/`,
but those generated files were only displayed on the website; nothing compiled or
ran them. JS and Python examples were already aggregated into runnable healthchecks
(`platforms/web/healthcheck/generated_examples.mjs` + `platforms/python/examples/main.py`);
Swift and Kotlin had no equivalent, so transpiler regressions and missing uniffi exports
went unnoticed until they reached the website.

`scripts/website.rs::write_healthcheck_aggregators` now also emits two compile-only
aggregators that embed every generated example body inside a private wrapper function:

- `platforms/swift/healthcheck/GeneratedExamples.swift`: picked up by the existing
  SPM executable target. `./healthcheck ios` was split into two sub-tests
  (`platforms.swift.bindings` + `platforms.swift.examples`); the second runs
  `xcodebuild -scheme fragmentcolor-healthcheck -destination 'generic/platform=iOS Simulator' build`
  and fails if any embedded body fails to type-check.
- `platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/GeneratedExamples.kt`,
  placed under `androidTest` so the existing
  `gradle fragmentcolor:connectedAndroidTest` invocation in `./healthcheck android`
  compiles the wrappers as part of the test source set. No CI YAML change needed.

Compile-only on day one; runtime execution requires a live GPU surface and is
deferred. The wiring stands on its own; runtime invocation can be promoted later
by uncommenting calls inside the `@Test` (Kotlin) or referencing the wrappers from
an `async` runner (Swift).

### Distribution

- Add root-level `Package.swift` for Swift Package Manager consumers. Users depend on the
  repo URL (`https://github.com/vista-art/fragmentcolor`, `from: "0.11.0"`); SPM resolves
  the tag, downloads the matching `fragmentcolor.xcframework.zip` from the GitHub Release,
  and verifies the pinned SHA-256 checksum.
- The Kotlin AAR is distributed through GitHub Releases for 0.11.x. Maven Central publishing
  (requires GPG + Sonatype OSSRH credentials) is tracked as follow-up work.

### Texture formats

- [x] Add `TextureFormat::Rgba16Float` (filterable + storage-writable in core WebGPU; no feature opt-in).
      Enables higher-precision iterative simulation (diffuse + transport + evaporate) without the
      8-bit storage precision loss, while avoiding `Rgba32Float` (unfilterable, requires
      `FLOAT32_FILTERABLE`) and `Rgba16Unorm` (requires `RGBA16UNORM_STORAGE`).

### Compute DX

- `Renderer::read_texture(texture_id) -> Vec<u8>` (plus `read_texture_async`) and
  `Texture::get_image` / `Texture::get_image_async` return the mip-0 contents of any registered
  texture as tightly-packed bytes in its native format. No more round-tripping through a
  `TextureTarget` and a fullscreen present to inspect a storage texture.
- `Renderer::create_storage_texture_with_data(size, format, bytes, usage)` creates a storage
  texture pre-seeded from a CPU blob in one call. Skips the "author a trivial seed WGSL shader"
  workaround for initial conditions. Expects tightly-packed bytes (no per-row padding) so small
  textures work without manually padding rows to 256.
- `Renderer::wait()` blocks until every queued submission on the device has finished.
  Restores deterministic ordering around `render()` → readback sequences. Previously a compute
  burst followed by `TextureTarget::get_image` could return stale pixels ~30-40% of the time on
  some Metal adapters because the readback raced the prior submission.
- Bind group layout now infers `TextureSampleType::Float { filterable: false }` when a sampled
  texture is only consumed by `textureLoad` (no `textureSample*` anywhere in the module). This
  unlocks formats like `Rgba32Float` as a sampled source without requiring the `FLOAT32_FILTERABLE`
  feature. The analysis is conservative: if any sample expression resolves through function
  arguments or other indirection we cannot statically follow, every image global is flagged as
  sampled (the pre-existing `filterable: true` layout).

### Bug fixes

- Sampled textures (`texture_2d<f32>`, `texture_2d<i32>`, `texture_2d<u32>`, multisampled, cube, 3D,
  arrayed, etc.) and samplers (filtering + comparison) now expose `ShaderStages::COMPUTE` in their
  bind group layouts, matching the visibility already granted to depth textures, uniforms, and
  read-only storage buffers. Previously compute shaders sampling a non-storage texture (or using
  a sampler to filter one) failed pipeline creation with "Shader global ResourceBinding ... is not
  available in the layout / Visibility flags don't include the shader stage", forcing workarounds
  like declaring sources as `texture_storage_2d<..., read>`.
- `texture_storage_2d<..., read_write>` is no longer silently downgraded to `ReadOnly`. Naga
  represents `read_write` as `StorageAccess::LOAD | STORE`, which previously fell through the match
  arm and produced a read-only bind group layout, so any `textureStore` then failed validation. The
  mapping now emits `StorageTextureAccess::ReadWrite` for the combined case, allowing ping-pong
  pairs to collapse to a single texture where the format supports it.
- `Texture::get_image_async` no longer deadlocks on native. The async path now drives
  `device.poll(Wait)` before awaiting the map callback; without it the oneshot future waits
  forever because nothing else advances the wgpu event loop on non-web targets.

### Platform workarounds

- Apple Silicon: on `macos` / `ios` targets the renderer now submits the current command buffer
  between two sequential compute passes, then records the next compute pass on a fresh encoder.
  The submission boundary reliably flushes Metal's tile-based storage-texture writes so a
  subsequent `texture_2d<f32>` / `textureLoad` in the next compute pass observes the results.
  Previously this pattern silently returned zeros. Compile-time routed via a new `apple` cfg
  alias (`target_os = "macos" | "ios"`); users do not need to opt in, and non-Apple targets are
  unaffected. The previous workaround (declaring the source as `texture_storage_2d<..., read>`)
  is no longer required.

### Transpiler: Rust-idiom scrubbing for Swift / Kotlin / JS / Python

The `docs/api/**.md` Rust examples are transpiled to four targets. The previous
output inherited Rust syntax in several cases, and Swift and Kotlin saw it loudest
once the new aggregators (above) compiled the per-doc output instead of just
displaying it. Cleanup pass driven by the principle "idiomatic to the target
language; cut or translate any Rust-specific idiom":

- **Multi-line method chains reassemble** before any per-line transform runs.
  `let x = r\n  .method()\n  .await?;` → `let x = await r.method();` rather than
  three orphaned lines that mangled `await`. Raw-string state tracked across the
  merge so WGSL inside `r#"..."#` is left intact.
- **Rust integer / float type suffixes stripped**: `0u8` → `0`, `64u32` → `64`,
  `100.0_f32` → `100.0`, `1isize` → `1`. Suffix probe only fires after a real
  digit run, so identifiers like `vec3<f32>` are not touched.
- **Rust unary deref dropped**: `renderer.read_texture(*texture.id())` →
  `renderer.read_texture(texture.id())`. Multiplication (`a * b`) and pointer
  types are not matched because both sides would be ident chars.
- **Rust array-repeat literal translates per language**: `vec![0u8; 256]` /
  `[0u8; 256]` becomes `Array(256).fill(0)` (JS), `[0] * (256)` (Python),
  `Array(repeating: 0, count: 256)` (Swift), `Array(256) { 0 }` (Kotlin).
- **Standalone `let var = r#"..."#`:** multi-line bare raw-string assignments
  (not just `Type::new(r#"..."#)`) are now gobbled and re-emitted as a single
  triple-quoted (Swift / Kotlin / Python) or backtick (JS) string. Used by the
  shader-composition example.
- **Single-quoted JS string literals → double-quoted** in Swift and Kotlin
  output. Outside-strings detection so apostrophes inside WGSL comments survive.
- **Kotlin `[a, b, c]` collection literals → `arrayOf(a, b, c)`** because Kotlin
  only accepts `[...]` syntax in annotation arguments. Indexer patterns
  (`arr[0]`) are not rewritten.
- **Source-level overrides** for examples that have no idiomatic Swift / Kotlin
  equivalent. Added per-language `hidden/<file>_<lang>.md` for
  `Renderer::create_external_texture_from_html_video` (wasm-Rust-only) and
  `Texture::set_sampler_options` (uniffi gap). Both render as a stub comment
  on the website.

After this pass the Swift aggregator parses cleanly under `swiftc -parse` (down
from 50 parse errors). Type-check / compile errors remain. The per-doc Rust
APIs still don't always map cleanly onto the uniffi-flattened Swift / Kotlin
signatures (e.g. Rust `[w, h]` `impl Into<Size>` vs uniffi
`createTextureTarget(width: UInt, height: UInt)`, `Shader::default()`
not exported, etc.). Tracked under _Carried over to 0.12.0_.

### Known issues

- The freshly-wired Swift / Kotlin doc-example aggregators (see _Doc-example healthchecks_
  above) surface a backlog of pre-existing transpiler bugs that the website was already
  shipping silently. The first round of fixes (above) reduced Swift parse errors from
  50 to 0; the remaining failures are type-check / compile errors that need either
  per-language emission with uniffi-signature awareness or source rewrites:
  - **`headless_window`-derived JS DOM leak:** Rust source uses `headless_window([w, h])`
    + `renderer.create_target(window)`, which today maps to `document.createElement('canvas')`
    (JS-specific). Swift / Kotlin output now parses (single quotes swapped) but `document`
    doesn't exist on those platforms. Either the source examples need to use
    `create_texture_target([w, h])` (portable across all four languages and uniffi-exported)
    or per-language `hidden/_swift.md` / `_kotlin.md` overrides need a CAMetalLayer /
    SurfaceView snippet.
  - **`Shader.default()` / `Shader.fromMesh(mesh)`** are not uniffi-exported; Swift / Kotlin
    examples reference methods that don't exist. Either expose them via uniffi or rewrite
    the source examples to use `Shader::new(source)`.
  - **Rust `[a, b]` `impl Into<Size>` vs uniffi flattened signatures:** many examples pass
    `[width, height]` arrays to methods like `createTextureTarget` whose uniffi-exported
    Swift / Kotlin signature takes positional `width: UInt32, height: UInt32`. Needs
    per-call-site detection in the transpiler, OR uniffi exports that accept an
    array-shaped `Size` struct.
  - 7 stale generated files under `platforms/{swift,kotlin}/examples/` (and `web/`,
    `python/`) left behind by the recent `docs/api` deletions (`update_texture.md`,
    `update_texture_with.md`, `write_with.md`, `texture_write_options/**`). The aggregator
    correctly excludes them but they linger on disk; platform-side cleanup of stale
    generated files is not yet implemented.
- Apple Silicon: the same TBDR-flush class of bug also manifests when a compute pass storage-writes
  a texture and a subsequent render pass in the same command buffer samples it. The render pass's
  `textureSample*` / `textureLoad` can observe zeros. The 0.11.0 auto-split covers
  `compute → compute` only; `compute → render` is not yet auto-split. Workaround: insert an
  explicit split between the two passes (for example, issue two `Renderer::render` calls, or call
  `Renderer::wait()` between them). Tracked on the roadmap for 0.12.x as an extension of the
  same `prev_was_compute` heuristic in the pass-dispatch loop.

### Dependency Updates

- Upgrade `wgpu` and `naga` from 27.0.1 to 29.0.1. Public fragmentcolor API is unchanged but the
  internal adapter was updated for every breaking change upstream shipped across 28.x and 29.0:
  - `wgpu::SurfaceError` was removed in favour of the `CurrentSurfaceTexture` enum. A local
    `fragmentcolor::SurfaceError` enum (re-exported at the crate root) replaces it with the same
    `Lost / Outdated / Timeout / OutOfMemory / …` variants, so downstream error enums and the
    `Target` trait keep a stable shape. A helper converts `wgpu::CurrentSurfaceTexture` back into
    `Result<SurfaceTexture, SurfaceError>` at every call site.
  - `InstanceDescriptor::default()` was removed; we now call
    `InstanceDescriptor::new_without_display_handle_from_env()`, which preserves the previous
    env-variable-driven configuration behaviour.
  - WGSL `var<push_constant>` is no longer accepted by naga's WGSL front end (only
    `var<immediate>`). Existing user shaders keep working: fragmentcolor rewrites
    `var<push_constant>` → `var<immediate>` before handing the source to naga.
  - `wgpu::Features::PUSH_CONSTANTS` → `Features::IMMEDIATES`; `Limits::max_push_constant_size` →
    `max_immediate_size`; `RenderPass::set_push_constants(stages, offset, data)` →
    `set_immediates(offset, data)` (stage argument dropped);
    `PipelineLayoutDescriptor::push_constant_ranges` → `immediate_size: u32`. The fallback path
    that rewrites oversized immediates into uniform buffers was updated for the new
    `naga::AddressSpace::Immediate` variant.
  - `RenderPassDescriptor` gained the `multiview_mask: Option<NonZeroU32>` field (set to `None`);
    `RenderPipelineDescriptor::multiview` was renamed to `multiview_mask`.
  - `DepthStencilState::depth_write_enabled` and `depth_compare` became `Option<…>` to allow
    explicit unset semantics (we now pass `Some(true)` / `Some(LessEqual)`).
  - `PipelineLayoutDescriptor::bind_group_layouts` is now `&[Option<&BindGroupLayout>]` to allow
    gaps; every call site builds a `Vec<Option<&BindGroupLayout>>` instead of a plain `Vec<_>`.
  - `SamplerDescriptor::mipmap_filter` now takes `MipmapFilterMode` rather than `FilterMode`.
  - `wgpu::Instance::new` now takes `InstanceDescriptor` by value (used to be `&_`).

### Shipped in 0.11.0

- [x] `platforms/swift/` Swift Package (SPM) + root `Package.swift` pulling xcframework from GitHub Release
- [x] `platforms/kotlin/fragmentcolor/` Android Library gradle module with `jniLibs` + generated Kotlin
- [x] `build_ios` script (build for `aarch64-apple-ios` + `aarch64-apple-ios-sim`, bundle xcframework)
- [x] `build_android` script (build all 4 Android ABIs via `cargo-ndk`, copy `.so` into `jniLibs`)
- [x] Mobile healthcheck scaffolding (Swift `Healthcheck.swift` + Kotlin `Healthcheck.kt`)
- [x] CI workflow: `healthcheck_ios.yml` + `healthcheck_android.yml` run on every PR
- [x] Release workflow: `publish_swift.yml` uploads xcframework, `publish_android.yml` uploads AAR
- [x] Post-release: `post_publish_update.yml` pins `Package.swift` to the released checksum
- [x] Swift + Kotlin doc transpilers generating per-language examples from every `docs/api` file
- [x] Swift + Kotlin doc-example aggregators wired into `./healthcheck ios` (split into
      `platforms.swift.bindings` + `platforms.swift.examples`) and the existing
      `connectedAndroidTest` flow (compile-only, mirroring the JS / Python coverage that
      already existed for the same per-doc transpiled output)
- [x] Compute DX suite: `Renderer::read_texture`, `read_texture_async`, `Texture::get_image` /
      `get_image_async`, `Renderer::create_storage_texture_with_data`, `Renderer::wait`
- [x] Bind-group-layout inference: `filterable: false` for textures only used via `textureLoad`
      (unlocks `Rgba32Float` as a sampled source without `FLOAT32_FILTERABLE`)
- [x] Compute-shader bind-group visibility: sampled textures + samplers now expose
      `ShaderStages::COMPUTE` alongside VERTEX/FRAGMENT
- [x] `texture_storage_2d<..., read_write>` correctly maps to `StorageTextureAccess::ReadWrite`
      (previously silently downgraded to read-only)
- [x] `Texture::get_image_async` native deadlock fixed (async path now drives `device.poll(Wait)`)
- [x] Apple-Silicon auto-split between sequential compute passes to flush TBDR storage writes
      (compile-time routed via the `apple` cfg alias)
- [x] `TextureFormat::Rgba16Float` (filterable + storage-writable in core WebGPU; no feature opt-in)
- [x] `wgpu` / `naga` upgrade from 27.0.1 to 29.0.1 (full adapter; see _Dependency Updates_)

### Carried over to 0.12.0

- [ ] Example iOS app under `platforms/swift/examples/` (xcodeproj consuming the SPM package)
- [ ] Example Android app under `platforms/kotlin/examples/` (gradle project consuming the AAR)
- [ ] Expand mobile healthchecks beyond the headless smoke test (textures, immediates, frames, …)
- [ ] Drain the Swift / Kotlin doc-example punch list surfaced by the new aggregators (see
      _Known issues_). Round 1 (parse errors) is done; Swift now parses 0 errors. Round 2
      (type / compile errors) needs per-language emission with uniffi-signature awareness:
      detect `createTextureTarget([w, h])` and rewrite to positional args, replace
      `headless_window` with `create_texture_target` at the source level (or per-language
      override), and either expose `Shader::default` / `Shader::from_mesh` via uniffi or
      rewrite the offending source examples. Then promote both aggregators from compile-only
      to runtime execution.
- [ ] Platform-side cleanup of stale generated examples under `platforms/{web,python,swift,kotlin}/examples/`
      when their source `docs/api/*.md` is deleted (today the website MDX is pruned but the
      per-platform sources are not)
- [ ] Implement `Renderer::create_external_texture` for real on every platform. The API
      surface now exists everywhere as a stub, but the actual mapping needs per-platform
      plumbing: Web `HTMLVideoElement` → `wgpu::ExternalTexture` (wgpu-web has the hooks),
      iOS `CVPixelBuffer` (via `CVMetalTextureCacheCreateTextureFromImage`),
      Android `SurfaceTexture` → `EGLImage` → external sampled texture. Keep the unified
      `createExternalTexture` entry point and add per-language extension shims that
      accept the platform-native source type and forward the underlying handle through
      uniffi (mobile) or wasm-bindgen (web).
- [ ] Publish Kotlin AAR to Maven Central (requires Sonatype OSSRH creds + GPG signing)
- [ ] Publish Swift Package to the Swift Package Index (register repo after first tag)
- [ ] Contribute struct-rename support to uniffi upstream (if ever needed for naming parity)
- [ ] Core helper `create_target_from_surface(surface, size)` to deduplicate Web/Python/iOS/Android
- [ ] Extend the Apple auto-split to cover `compute → render` sampled-read hazards (same TBDR
      class as the already-handled `compute → compute` case; see _Known issues_ above)
- [ ] Revamp RenderPass API (expose all `wgpu::RenderPass` customizations with sensible defaults)
- [ ] Specialized alias objects (`Compute`, `RenderPass`, `ComputePass`)
- [ ] Custom blending

## 0.10.11 Texture write API, renderer texture updates, and external video textures

### API additions

- Add `Texture::id`, `Texture::write`, and `Texture::write_with`.
- Add `Renderer::update_texture`, `Renderer::update_texture_with`, and `Renderer::unregister_texture`.
- Add public `TextureWriteOptions` to control upload origin, size, bytes-per-row, and rows-per-image.

### Web

- Add `Renderer::create_external_texture_from_html_video` for sampling HTML video via `texture_external`.

### Python / RenderCanvas

- Restore compatibility with RenderCanvas versions that removed custom string-based `get_context("fragmentcolor")` integrations.
- `Renderer::create_target` no longer depends on a custom RenderCanvas context name; it now reads screen present info directly, creates the surface from that data, and reuses the cached `RenderCanvasTarget`.

### Docs & generation

- Add docs and generated examples for the new texture-write APIs across Rust, JavaScript, and Python.
- Support language-specific override snippets when the generic Rust-to-JS/Python conversion is insufficient.

## 0.10.10 Web glue guard, ArrayBuffer handling fixes, and website hero cleanup

### Build system

- Web (WASM): add a post-bindgen patch step in `build_web` that hardens the generated glue.
  - Guard the `Uint8Array(ArrayBuffer)` constructor used by wasm-bindgen shims against detached ArrayBuffer.
  - On failure, fall back to `new Uint8Array(wasm.memory.buffer)` (live memory) to avoid crashes in long prod runs.

### Web (WASM)

- TextureInput (JS bridge): make ArrayBuffer handling robust on Web; treat `byte_length() == 0` as detached/empty and return an empty byte vector instead of throwing.
- ImageData/Canvas extraction: use `ImageData.data().0` (Clamped<Vec<u8>>) for efficient copies; remove incorrect `copy_to()` usage on clamped data.

### Docs & website

- ShaderHero: simplify the component and remove heavy per-frame instrumentation/noise; keep a concise render loop and error stop.

### Tooling & misc

- Update `run_docs`, `astro.config.mjs`, and lockfiles for the site and examples.


## 0.10.9 Bugfix: Stable kind branding for JS (avoids mangling in minified builds)

### Bugfixes

- Stable kind branding for JS (avoids mangling in minified builds)
  - `crate::impl_fc_kind!(TypeName, "TypeName");` in each type's file
  - `pub mod fc_kind;` and `pub use fc_kind::FcKind;` in lib.rs

## 0.10.8 Concurrency‑safe uniforms, typed errors, and web gallery

### API changes

- RendererError: add `MsaaViewMissing` and `DepthSampleCountMismatch`; InitializationError: `AdapterNotSet`.
- ShaderError: add `Busy`. `set()` is now non‑blocking (queues last‑wins updates); read methods may transiently return `Busy` under contention.

### Internals

- Non‑blocking uniform updates with pending queue; renderer flushes pending before binding.
- Pass adopts kind (Render/Compute) from the first attached shader.
- Web (WASM): pre‑grow linear memory by 64 MiB to reduce mid‑frame `memory.grow` stalls.

### Bug fixes

- JavaScript: fix "Invalid target type in render" in website by shipping branded JS prototypes in the npm package.
- Web gallery/healthcheck: use `init({ module_or_path })` for reliable WASM initialization.

### Docs & website

- Replace old healthcheck pages with a Visual Gallery at `/gallery`; default `run_web` to Gallery.
- Homepage: add `ShaderHero` and tighten hero spacing; Astro now points to local pkg dir so subpath imports resolve.

### Examples

- Rust: `swirl` shader moved to `examples/rust/examples/shaders/swirl.wgsl`; example loads by path and uses top‑level `draw`/`resize`.
- Web: simplify pass dependencies and update to `createTextureTarget()` API.

## 0.10.7 Documentation automation, website integration, API completeness, build system, and release flow

This is our biggest release to date and it feels extremely weird to merely bump a patch version!

The amount of changes introduced here deserves a major version bump!

Most of the following features were originally planned for future releases, but I get carried away
and implemented them now, focusing on API completeness before tackling iOS and Android support.

### Geometry/Instancing Refinement

- [x] Per-mesh bind-group updates to allow different textures per mesh in a single pass.
- [x] Mesh-Shader compatibility enforcement and Shader-centric mesh attachment API (Shader.add_mesh, Pass.add_mesh_to_shader).
- [x] Geometry/Instancing Refinement:
  - [x] Shader source-derived validation/mapping of @location inputs from Naga
  - [x] Multiple meshes per Pass and per-mesh draw calls.
  - [x] Design a Idiomatic/simple way to create complex shapes with Mesh and Vertex.
  - [x] Mesh builders for common shapes (only Quad for now)
  - [x] Meshes grouped by Shader; multiple Pipelines per Pass with multiple meshes.

### Shader API Completeness

- [x] Support for all ways to upload data to a GPU:
  - [x] VertexBuffer
  - [x] IndexBuffer
  - [x] StorageBuffer
  - [x] StorageBuffer: Arrays
  - [x] Uniform
  - [x] Uniform: Arrays
  - [x] Texture
  - [x] StorageTexture
  - [x] Sampler
  - [x] PushConstant

### Complete Texture and Storage API

#### Core types

- [x] Rename internal renderer/texture.rs::Texture -> TextureObject with sampler options (RwLock)
- [x] Public API: Texture wrapper (Arc\<TextureObject\>, TextureId handle), add Arc\<RenderContext\>
- [x] Handle registry in RenderContext (DashMap\<TextureId, Arc\<TextureObject\>\>) + AtomicU64 allocator
- [x] Introduce TextureId newtype to avoid conflict with TexturePool, keep TexturePool as-is
- [x] Introduce TextureMeta (id + naga metadata: dim, arrayed, class)

#### Shader UX

- [x] UniformData::Texture carries TextureMeta; From<&Texture> sets id only (preserves shader metadata)
- [x] UniformData::Storage((inner, span, access)) with CPU-side blob backing. set("ssbo.*") updates the blob at field offsets (from Naga) and renders on next frame.
- [x] JS/Python conversions to allow shader.set("key", texture)
- [x] Naga parsing: detect image/sampler bindings; store TextureMeta/SamplerInfo in UniformData
- [x] AddressSpace handling: accept Uniform/Handle; WorkGroup (unbound) ignored; PushConstant supported natively (single-root) with uniform-buffer fallback (Web or over limit/multi-root).
- [x] Array element indexing for Storage and Uniforms using naga stride, including nested array/struct offsets. Added unit tests.
- [x] Unified cross-target URL fetching helper (native via ureq, WASM via fetch) and refactored Shader.fetch and texture URL loading to use it. Removed ureq usage from WASM paths.
- [x] Texture & Sampler support

#### Renderer bindings and draw

- [x] Bind group layout: add Texture (sampled/depth/storage) and Sampler entries using Naga metadata
- [x] Storage buffers: bind as BufferBindingType::Storage/ReadOnlyStorage based on access flags; upload CPU blob each frame via a dedicated pool
- [x] Render: bind TextureView and Sampler (resolve from TextureId); fallback/default sampler if needed

#### Unified input and ergonomics

- [x] TextureInput enum (Bytes, Path, CloneOf); From impls for &[u8], Vec<u8], &Path, PathBuf, &Texture
- [x] TextureOptions (size, format, sampler) with conversions from Size/TextureFormat; TextureFormat wrapper decoupled from wgpu
- [x] create_texture prefers encoded bytes or path; use create_texture_with for raw bytes + options
- [x] Aliases: create_2d_texture / create_3d_texture / create_storage_texture / create_depth_texture

#### Cleanup

- [x] Remove all Box<dyn Error>, created module-scoped error types

### Rendering

#### Renderer Internals

- [x] Surface configuration selection and view_formats
- [x] Surface frame acquire recovery (WindowTarget)
- [x] Sample-count negotiation helper
- [x] Store and propagate sample_count in RenderContext
- [x] Pipeline cache keyed by (ShaderHash, format, samples)
- [x] MSAA render path with resolve
- [x] TextureTarget MSAA + resolve (optional)
- [x] Centralized frame acquire retry in Renderer
- [x] Pooling for transient targets/readback

#### Renderer API

- [x] create_texture(input: Into\<TextureInput\>) -> Texture (Rust)
- [x] create_texture_from_file(&Path) -> Texture (Rust)
- [x] create_texture_with(input, options: TextureOptions) -> Texture (Rust); alias helpers: create_texture_with_size, create_texture_with_format
- [x] Web: createTexture(input) (Uint8Array/URL/query selector)
- [x] Python: create_texture(input) (bytes/path/ndarray)

### Build System and Documentation

- [x] Build System
  - [x] Unit test all packages before building
  - [x] Git hook: test builds for all platforms before push
  - [x] Script to Test, Compile & Publish JS
  - [x] Script to Test, Compile & Publish Python
  - [x] Script to Test, Compile & Publish Rust + Winit
  - [x] GHA wheel: Test build all packages for all OSses

- [x] Automated documentation pipeline:
  - [x] Doc strings centralized in `docs/api` and consumed in Rust via `#[lsp_doc]`.
  - [x] Build-time validation: ensures object/method docs exist and include a `## Example` section.
  - [x] Website generator: converts `docs/api` into MDX pages under `docs/website`, downshifting method headings and stripping object H1.
  - [x] JS/Python examples are sliced from annotated healthcheck scripts.

- [x] Release Management & Website Automation
- [x] Website moved into this repository under `docs/website`.
- [x] Automatically update docs from Rust Doc Comments
- [x] Script to copy contents and publish to Website
- [x] Post-publish workflow: after tags publish to npm & PyPI, update consumers (website & JS example) to the released version and push to main.
- [x] Healthcheck example markers add

### Platforms

- [x] iOS/Android scaffolding: platform wrappers and targets aligned with Python/JS method order (bindings not generated yet)
  - [x] Renderer: `new_ios`, `create_target_ios`, `create_texture_target_ios`, `render_ios`
  - [x] Renderer: `new_android`, `create_target_android`, `create_texture_target_android`, `render_android`
  - [x] Types: `IosTarget`, `IosTextureTarget`, `AndroidTarget`, `AndroidTextureTarget`

## Documentation and Tooling

- [x] Aggressive build system that does a lot of magic
- [x] 100% feature parity guaranteed across all languages
- [x] Normalized API links to <https://fragmentcolor.org>.
- [x] Wired all public items to `#[lsp_doc]` sources (Renderer, Shader, Pass, Frame, etc.).
- [x] Docs examples standardized: async wrapper + pollster, no futures dependency, no filesystem reads. Prefer create_texture in examples; use create_texture_with only on its own page or for raw-byte cases.
- [x] Removed inline JS/Python examples on core API pages (these are generated elsewhere). Hidden platform docs left intact.
- [x] Removed stale mobile code paths: `headless()`, `render_bitmap()`, and platform `FragmentColor` wrappers.
- [x] Moved platform-specific cfgs out of `renderer/mod.rs`; added `renderer::platform::all::create_instance()` and moved the winit `HasDisplaySize` impl to `renderer/platform/winit.rs`.
- [x] build.rs validation: ignore mobile wrapper variants (`*_ios`, `*_android`) just like `*_js` and `*_py` when mapping docs.

## 0.10.6 JavaScript support (skipped released due to unstable build)

- [x] Adds JavaScript support
- [x] Publishes to NPM
- [x] Refactor to remove constructors returning tuples
- [x] Lazy-loaded Renderer with easier API
- [x] Chore: Script to automatically bump version
- [x] Updates WGPU version and other dependencies

## V 0.10.5 Fixes Python support for Windows and Linux

- [x] Fixes Python support for Windows and Linux
- [x] Automatically generate artifacts
- [x] Automatically publish to Pypi

## V 0.10.4 Fixes Python Import Error

- [x] Fixes Python import in Pypi distribution
- [x] Fixes Pass and Frame objects
- [x] Adds missing methods from public API

## V 0.10.3 Rust 2024 Edition

- [x] Upgrades to Rust 2024 edition
- [x] Minor fixes in documentation

## V 0.10.2 Python support

- [x] Initial Python Draft Implementation
- [x] Publish to Pip

### V 0.10.1 Cleanup and Fix Bugs

- [x] Simplify Shader Internal representation
- [x] BufferPool implementation
- [x] Graceful runtime error handling (no panics)
- [x] Fix uniform getter and setter
- [x] Renderer render() method now has two arguments: Renderable and Target
- [x] Make the Renderer support Shader, Pass and Frame as input
  - [x] Shader
  - [x] Pass
  - [x] Frame
- [x] Improve public interface for Target - make the default easy (one target)
- [x] Set up cross-platform initializers (helper functions)
- [x] remove boilerplate from Rust demos

### V 0.10.0 Set up basic API and basic data structures

- [x] Renderer
- [x] Shader
  - [x] decide how to handle generic set() function
        [x] Pass
  - [x] RenderPass
  - [x] ComputePass
- [x] Frame
- [x] Renderer
- [x] Target
- [x] Design main public interface
- [x] Experimental GLSL Support

## Earlier Versions (before V 0.9.0 in 2023)

The initial versions of this library (up tp 0.9.0) were completely discarded.

About one year after not touching the code, in January 2025 I force-pushed and rewrote the **v0.10.0** branch from scratch.

---

## TEMPLATE

### Fixed

- Bumps patch version

### Added

- Bumps minor version

### Deprecated

- This is unused until the initial public release. While in prototyping stage, the API is a hot mess and can change anytime in a whim.

### Removed

- Bumps major version
