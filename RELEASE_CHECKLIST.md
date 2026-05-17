# v0.11.2 Pre-Release Checklist

Punch list captured from the branch review on 2026-05-15 and worked
through on 2026-05-16 / 2026-05-17. Everything checked below has landed
on the branch; everything unchecked is queued for a follow-up cycle.

Baseline at the end of the 2026-05-17 polish pass: **262 lib tests, 157
doctests, 3 integration tests** all passing, clippy clean.

Severity ladder:

- **Block** — must land before tag.
- **Polish** — land if a "release polish" sweep is in scope; otherwise
  carry into the next minor.
- **Defer** — performance / refactor not required for correctness;
  schedule against a follow-up cycle.

## Done in this branch

### Storage readback + wgpu encapsulation sweep (2026-05-16)

Driven by the differentiable-renderer prototype, which needed to read
GPU-written gradients out of a storage buffer without using the
`Rgba32Float` storage-texture trick. Same commit cycle closed every
remaining `wgpu::*` leak on the public API.

- [x] **`Renderer::read_storage(shader, binding).await -> Vec<u8>`** —
      GPU→CPU round-trip for any storage binding the shader declares.
      Mirrors `Texture::get_image()`'s shape (async, pooled readback
      buffer, raw bytes the caller `bytemuck::cast_slice::<T>`s).
      `src/renderer/mod.rs`, new `src/renderer/storage_read.rs`,
      `docs/api/core/renderer/read_storage.md` (+ Web / Python / mobile
      hidden mirrors). Platform bindings — `readStorage` (JS),
      `read_storage` (Python), `readStorage` (Swift / Kotlin) — share
      the same doc page.
- [x] **`RendererError::StorageBindingNotFound(name)`** typed error
      when the shader doesn't declare the binding or no render pass has
      materialised the GPU buffer yet. `src/renderer/error.rs`.
- [x] **`Mipmap::format` no longer leaks `wgpu::TextureFormat`** —
      returns `crate::TextureFormat` (the existing public enum) via the
      already-present `From` impls. `src/texture/mod.rs`.
- [x] **`TextureUsage` introduced as the public bitflag mirror of
      `wgpu::TextureUsages`** with named constants (`COPY_SRC`,
      `COPY_DST`, `TEXTURE_BINDING`, `STORAGE_BINDING`,
      `RENDER_ATTACHMENT`), `BitOr` / `BitOrAssign`, `contains`, and
      bidirectional `From<wgpu::TextureUsages>`. Stored as `u32` so it
      crosses every FFI cleanly. `TextureOptions::with_usage` now
      takes the public `TextureUsage` instead of `wgpu::TextureUsages`.
      `src/texture/usage.rs`, `src/texture/options.rs`.
- [x] **`Region::from_wgpu_size` / `to_wgpu_size` removed.** Had no
      callers anywhere in the tree — pure dead code with a wgpu leak.
      `src/region.rs`.
- [x] **`pub type Commands = Vec<wgpu::CommandBuffer>` removed.**
      Same as Region — no callers, pure wgpu leak. `src/renderer/mod.rs`.
- [x] **`TargetFrame` + `Target::get_current_frame` sealed via a
      `pub(crate) trait TargetInternal` supertrait.** External code can
      still use `&impl Target` for `size` / `resize` / `get_image`, but
      can no longer implement `Target` for their own types — all
      wgpu-touching frame-acquisition lives behind the sealed bound.
      `TargetFrame` itself is now `pub(crate)`, so `wgpu::TextureView`
      and `wgpu::TextureFormat` no longer leak through its methods.
      Affected impls: `TextureTarget`, `WindowTarget`, `RenderTarget`,
      `CanvasTarget`, `RenderCanvasTarget`, `PyTextureTarget`,
      `MobileWindowTarget`, `MobileTextureTarget`, plus the renderer's
      `DummyTarget` test fixture. `src/target/mod.rs`,
      `src/target/{texture,window,all}.rs`,
      `src/target/platform/{web,mobile,python}.rs`,
      `src/renderer/mod.rs`.
- [x] **`surface_texture_from(wgpu::CurrentSurfaceTexture)` lowered to
      `pub(crate)`** — internal codec helper, never meant for outside
      callers. `src/target/mod.rs`.

After this pass the public API surface no longer mentions `wgpu::*` in
any function signature, return type, or field of a `pub` item. The
single remaining `wgpu` symbol on the rustdoc surface is the `wgpu::*`
re-export that internal users (or anyone reaching for the escape hatch
via fork) can still pull from — no method demands it.

### Release blockers (v0.11.2-dev review, day 1)

- [x] Fix wasm/Python/mobile build break — platform mirrors call
      `set_texture_or_warn` (eager wrapper) which delegates to
      `queue_texture_or_warn` with the canonical sRGB hint per PBR
      slot. `src/material/mod.rs`. *(Helper later removed when the
      platform mirrors switched to calling the public Rust setters
      directly — see below.)*
- [x] Stop leaking `glam::Vec4` through `IntoVertexPositionFull` —
      trait is now `pub(crate)` and its method returns `[f32; 4]`.
      `src/mesh/vertex.rs`.
- [x] Move `pub fn run(app: &mut App)` into `impl App` as
      `App::run(&mut self)`. All examples + the rust-with-app tutorial
      now call `app.run()`. `src/app.rs`,
      `examples/rust/examples/*.rs`,
      `docs/guides/building-apps/rust-with-app.md`.

### API consistency

- [x] **Material setter chaining unified.** Rust setters changed from
      `self -> Self` to `&self -> Self` (Arc-cloned handle, same shape
      Light and Camera use). Every platform mirror (Python / JS / Swift /
      Kotlin) now returns the matching `Self` / `Arc<Self>`, so chains
      like `Material::pbr()?.base_color(...).metallic(...)` work
      identically across languages. The `set_texture_or_warn` wrapper
      is gone — platform mirrors call the public Rust setters directly.
- [x] **`Pass::add_target` → `Pass::set_target`** (and
      `add_depth_target` → `set_depth_target`) — the methods overwrite,
      they don't accumulate. Lib, platform mirrors (`addTarget` →
      `setTarget` in JS / Swift / Kotlin), examples, tests, and doc
      pages renamed together. The `docs/api/core/pass/add_target.md`
      / `add_depth_target.md` files have been moved with `git mv` to
      preserve history.
- [x] **Camera vocab unified on `position`.** `Camera::look_at(eye,
      target, up)` → `look_at(position, target, up)`. Matches
      `Light::point(position, ...)` and `Camera::position()`. Doc page
      and all three platform mirrors updated.

### Doc fixes (mechanical sweep across 28 files)

- [x] `Material::pbr` no longer claims to take `&Renderer` —
      `docs/api/scene/material/pbr.md`, `material.md` describe the
      1×1 lazy fallback truthfully.
- [x] Six broken `Material::add` cross-links retargeted to `Pass::add`
      / `Scene::add` (camera/light pages and `pbr.md`).
- [x] `docs/api/scene/light/directional.md` no longer claims point /
      spot are "not in the MVP yet" (they ship today).
- [x] `Scene::scene.md` methods table now lists `ambient` and `load`.
- [x] `Light::light.md` methods table split into constructors /
      methods sub-tables.
- [x] `Model::material.md` and `Model::new.md` agree on the Arc-share
      semantics.
- [x] "Follow-up" / "deferred" / process-language stripped from
      `material/material.md`, `material/uv_transform.md`,
      `material/normal_texture.md`.
- [x] Doctest discipline: hidden `# ` lines bundle at the boundaries;
      visible `let _ = x;` no-ops gone from `texture/mipmap/*.md` and
      `camera/view_proj.md`. Light `spot.md` / `point.md` confirmed
      already compliant.
- [x] Model anchor casing (`#modelset_transform` → `#set_transform`)
      fixed in `rotate.md`, `transform.md`, `set_transform.md`.
- [x] `let mut mesh = Mesh::new();` → `let mesh` in the three
      `Model::*.md` pages where `mut` was dead.
- [x] `Material::pbr().unwrap()` in `base_color_texture.md` switched
      to `?`.
- [x] Module-path imports unified on top-level
      (`fragmentcolor::{Mesh, Vertex}`) in
      `geometry/mesh/{set_indices,clear_indices}.md`.
- [x] `building-apps/README.md` apologetic line rephrased.
- [x] Side fixes: dead `let renderer = Renderer::new();` lines
      stripped from doctests where the prose previously claimed
      `Material::pbr` took a `Renderer`.

### Renderer + scene safety

- [x] **Default Light direction** flipped from `[0.3, -1.0, -0.4]`
      to `[0.0, -0.3, -1.0]` so a hello-world `Scene` with the default
      `Camera` (eye at `+Z=5`) lights the canonical `+Z`-facing quad
      visibly. `src/scene/scene.rs`.
- [x] **`Camera::set_aspect` on orthographic** no longer log-warns
      and bails — it now keeps the vertical extent and rescales the
      horizontal extents around the existing horizontal midpoint, so
      a resize handler behaves identically across both projection
      kinds. The `Projection::Orthographic` enum variant carries its
      frustum params again. `src/scene/camera.rs`,
      `docs/api/scene/camera/set_aspect.md`.
- [x] **`Camera::set_aspect` doc** rewritten to describe both
      projection paths.
- [x] **`App::add_state` overwrite warning** — logs a `warn!` when
      `insert` replaces an existing entry.
- [x] **`App::get_state` type mismatch warning** — logs a `warn!`
      when the key exists but `downcast::<T>` fails.
- [x] **8-light cap → typed `PassError::LightCapReached`** in
      `Light::attach`. Pre-checks every shader on the Pass before
      attaching, mirrors the dedup logic, returns `Err` instead of
      silently dropping a 9th Light. `src/pass/error.rs`,
      `src/scene/light.rs`.
- [x] **`KHR_texture_transform` non-base-color warnings** — when a
      glTF file ships per-map transforms on `metallic_roughness` or
      `emissive` (the slots reachable through `gltf::texture::Info`),
      the loader logs a `warn!` instead of silently identity-rendering.
      The gltf crate doesn't expose `texture_transform()` on
      `NormalTexture` / `OcclusionTexture`, so those two slots are
      noted in the helper comment and not warned at the source.
- [x] **`Scene::load` partial-fail tolerance** — unsupported image
      pixel formats now log a `warn!` and skip the slot (the Material
      keeps its 1×1 default) instead of aborting the whole load. Hard
      errors (missing image, mismatched byte counts) still bubble.
      `src/scene/loader.rs`.
- [x] **`Scene::load(SceneSource::gltf(Path(...)))` on WASM** returns
      a typed `SceneLoadError::Invalid` explaining the limitation
      instead of compiling against `std::fs` and panicking.
      `src/scene/loader.rs`.
- [x] **Deterministic opaque draw order.** `build_pass_draws`
      stores `(shader, mesh)` groups in a `Vec<((K,V))>` walked in
      insertion order rather than a `HashMap` with random
      iteration. `src/renderer/mod.rs`.
- [x] **glTF Material dedupe per-mesh.** A `HashMap<Option<usize>,
      Material>` keyed by `gltf::Material::index()` is threaded through
      `visit_node`, so a glTF file with N primitives sharing one
      source material allocates one shader, not N.
      `src/scene/loader.rs`.

### Dead-code cleanup

- [x] `Pass::_storage_alias` removed (`src/pass/mod.rs`).
- [x] `Pass::present_to_target` removed (`src/pass/mod.rs`).
- [x] `BlendDraw.mesh_ptr` + `BlendDraw.shader` removed
      (`src/renderer/mod.rs`). The `mesh: Arc<MeshObject>` on
      `BlendDraw` keeps the mesh Arc alive across the render pass;
      the shader Arc lives in `pass.shaders` for the duration of the
      draw, so no extra clone is needed.
- [x] `CameraSnapshot.position` removed (`src/pass/mod.rs`,
      `src/scene/camera.rs`). The renderer reads only `.view` for the
      transparency sort.
- [x] `Material.alpha_mode` + `Material.double_sided` Arc-RwLock
      fields removed — the `ShaderObject` already carries them and
      the renderer reads only from there. All setters lost their
      double-write across the Rust lib and the three platform
      mirrors.
- [x] `Vertex::POSITION` constant removed — `vertex.set(Vertex::POSITION,
      ...)` was writing to a property bag the loader doesn't read.
      Position is set exclusively via `Vertex::new(...)`.
      `src/mesh/vertex.rs`, `docs/api/geometry/vertex/vertex.md`.
- [x] `decompose_trs` inlined at its two call sites
      (`src/scene/loader.rs`).
- [x] `map_alpha_mode` inlined (the 3-arm match) at its single call
      site (`src/scene/loader.rs`).

### Renderer modernisation (2026-05-17)

Landed by a dedicated renderer-perf agent on 2026-05-17. All five
items from the v0.12 "Defer" list shipped in one focused pass.

- [x] **Camera snapshot is computed once per propagate.**
      `propagate()` and `SceneObject::attach` each acquire the state
      lock once, compute `(view, view_proj, position)` together, and
      reuse them for the shader push and the Pass snapshot stamp.
      Cuts per-propagation lock acquisitions from three to one.
      `src/scene/camera.rs`.
- [x] **Per-Mesh AABB + transparency eye-Z uses centroid.**
      `MeshObject` carries `aabb_min` / `aabb_max` (`pub(crate)`),
      grown by `add_vertex`, exposed via `aabb_local()`. The
      transparency sort multiplies the centroid through `view *
      transform` instead of the model origin, so an elongated
      translucent beam centred away from origin sorts in the right
      order against a co-located cube. Public API unchanged — the
      AABBs are internal. `src/mesh/mod.rs`, `src/renderer/mod.rs`.
- [x] **One `ModelState` lock per Model.** Folded `transform` and
      `visible` into `Arc<RwLock<ModelState>>`. The renderer's
      per-Model loop halves its lock count. Existing scene tests
      (clone-shares, pass-entry-shares-visibility,
      hidden-model-doesn't-render, live-transform-after-add) updated
      and passing. `src/scene/mod.rs`.
- [x] **Renderer scratch caching + `bytemuck` pack.**
      `RenderContext` carries a `RenderScratch { opaque_groups }`
      under `RwLock` that `build_pass_draws` clears-and-reuses
      across frames instead of re-allocating. Per-Model matrix
      byte-pack collapses from 16 `to_le_bytes` calls to a single
      `bytemuck::cast_slice(&matrix.to_cols_array_2d())` slice copy
      for both opaque and blend draws. `src/renderer/mod.rs`.
- [x] **Global cross-shader blend sort.** Pre-loop snapshot of each
      shader's `(pipeline, bind_groups, immediate bytes)` into a
      `HashMap<usize, ShaderRenderState>`. The blend phase now walks
      `blend_draws` globally in eye-Z order, switching shader state
      only when `shader_ptr` changes. Two translucent Materials in
      the same Pass now interleave correctly back-to-front. New
      integration test `cross_shader_blend_sorts_globally` asserts
      blue-over-red at centre pixel for six scrambled-order quads
      across two Materials. `src/renderer/mod.rs`,
      `tests/blend_transparency.rs`.

### Light unification + cap bump (2026-05-17)

A short-lived three-type Light split (`DirectionalLight` /
`PointLight` / `SpotLight`) landed earlier in the day and was reverted
the same day. The library author's clarified design principle: **there
is one method per language to add things to a `Scene` or `Pass`**.
Concrete types implement `SceneObject`; the cross-language bindings
expose a single dispatching `add(...)` that branches on the runtime
type. Per-kind `addDirectionalLight` / `addPointLight` / `addSpotLight`
(plus the pre-existing `addModel` / `addCamera`) all collapsed into one
`add` dispatcher per platform.

- [x] **Unified `Light` type with kind-tagged constructors.**
      Single `pub struct Light { object: Arc<LightObject> }`. Three
      constructors: `Light::directional(direction, color)`,
      `Light::point(position, color)`, `Light::spot(position,
      direction, color)`. `LightKind` enum public for runtime
      inspection via `light.kind()`. Mirrors the `Pass::compute(name)`
      pattern (constructor on the unified type, not a separate type).
- [x] **`LightError` typed enum.** `FieldNotApplicable { kind, field
      }` for kind-specific setters called on the wrong kind;
      `NegativeRange(f32)` for `set_range(value < 0.0)`.
- [x] **Kind-aware setters and getters.**
      - Universal (return `Self`): `set_color`, `set_intensity`.
      - Kind-specific (return `Result<Self, LightError>`):
        `set_position`, `set_direction`, `set_range`,
        `set_cone_angles`.
      - Kind-specific getters return `Option<T>` (`position`,
        `direction`, `range`, `inner_cone_angle`,
        `outer_cone_angle`) — `None` on the kind that doesn't apply.
- [x] **Cap bumped 8 → 32.** `PBR_MAX_LIGHTS = 32` in
      `src/scene/light.rs` and `array<Light, 32>` in
      `src/material/pbr_main.wgsl`. The typed
      `PassError::LightCapReached { cap: 32 }` still fires on the
      33rd Light; the existing cap test was updated.
- [x] **One `add(...)` per binding for Scene and Pass.** Every
      `addModel` / `addCamera` / `addDirectionalLight` /
      `addPointLight` / `addSpotLight` method on the Python / JS /
      Swift / Kotlin wrappers collapsed into a single dispatching
      `add(...)`. Mobile uses a new `SceneObjectHandle::{Model,
      Camera, Light}` uniffi enum. Python casts `Py<PyAny>` through
      each concrete type. Web does the same via `JsValue::try_from`
      bridges. The Rust generic `add<O: SceneObject>` stays the
      single source of truth.
- [x] **Doc reorganisation.** 32 files under three `*_light/`
      subdirectories deleted; 18 files under flat
      `docs/api/scene/light/` created (light.md overview +
      kind.md + 3 constructor pages + 6 getter pages + 6 setter
      pages). The overview documents the cost model:
      forward-shaded, O(N) per fragment per light, 32-light cap fits
      the forward path comfortably, clustered / storage-buffer path
      is on the roadmap. `docs/api/PARITY_BASELINE` shrunk back to
      its pre-split state (one unrelated entry).

### Polish pass + deferred-items review (2026-05-17 evening)

After the unification settled, a focused polish + recommender pass:

- [x] **Clippy clean** under `cargo clippy --all-targets`. Fixes:
      `AlphaMode` now derives `Default`; collapsible `if let`s
      flattened in `material/mod.rs` and `scene/loader.rs`;
      `is_multiple_of` replacing manual `% 4 != 0`; doc-list
      indentation on `texture/options.rs`; build-script collapsible
      match arm in `scripts/codegen.rs`; `module_inception` allowed
      on `src/scene/scene` (deliberate flat layout); five test
      `pass.add(&camera)` sites now `expect("...");`;
      `create_render_pipeline` gets `#[allow(clippy::too_many_arguments)]`.
- [x] **Per-Shader Light cap documented + tested.**
      `docs/api/scene/light/light.md` gained a "Cap semantics" section
      explaining that the 32-cap binds to the underlying
      `ShaderObject`, not the `Pass`. Two Passes that share a Material
      share its light slots — Light X attached to Pass A and Light Y
      attached to Pass B end up in slots 0 and 1 of the shared
      shader, both visible to both Passes. A new lib test
      `cap_is_per_shader_not_per_pass` pins the contract.
- [x] **Material doc cleanup.** Eight pages
      (`metallic.md`, `normal_scale.md`, `alpha_cutoff.md`,
      `occlusion_strength.md`, `alpha_mode.md`, `shader.md`,
      and partial work on the four texture pages) had dead
      `let renderer = Renderer::new();` lines left over from when
      `Material::pbr` took a `&Renderer`. The unrelated `async fn run`
      wrappers got dropped too — these examples are all sync.
- [x] **`alpha_mode.md` accuracy fix.** Stale claim that
      cross-Material blend interleaving "falls back to per-shader
      sort" — corrected to reflect today's global cross-shader sort
      (Agent A's renderer modernisation work).

### Inline doc polish (2026-05-17)

Small doc improvements written alongside the agent work.

- [x] **Material Arc-share callout** on `docs/api/scene/material/material.md`
      (canonical "Cloning is an Arc-share" section) with a short
      pointer on `base_color.md`. Other setters defer to the
      canonical page rather than repeating 15× — minimal surface,
      single source of truth.
- [x] **Texture setter lazy-error semantics** documented on
      `base_color_texture.md` (the canonical PBR-texture page).
      Explains that setters are infallible at call time; the lazy
      path's errors surface when the queue drains (`Renderer::load(&material).await`
      or first render).
- [x] **`Scene::add_pass` clears by default.** New callout on
      `docs/api/scene/scene/add_pass.md` explaining that each new
      Pass starts with `PassInput::clear(transparent)` — chain via
      `pass.load_previous()` or `pass.set_clear_color(...)` to
      compose two passes through the same target.
- [x] **`create_external_texture.md` work-process leak fixed.** The
      "implementation is a follow-up" anti-pattern stripped from
      both the public and hidden versions.
- [x] **`Scene::load.md` Threading + WASM section.** Documents the
      synchronous nature of `gltf::import_slice`, recommends a
      worker thread for large `.glb` files on both native and WASM.

## Won't fix this cycle (kept for context)

- **`SceneSource::Gltf` single-variant collapse.** The `SceneSource`
  enum was flagged as speculative future scaffolding. After
  re-examination, the format tag is load-bearing for the `Bytes`
  variant (the gltf crate can't infer format from raw bytes —
  `import_slice` is `.glb`-only). Keeping the explicit tag means
  `Scene::load(SceneSource::gltf(bytes))` says what it does instead of
  "trust me to guess". Adding USD / FBX later is a non-breaking
  variant addition. No change.

## Polish — remaining

- [ ] **`Material::fork()` (deferred).** The Arc-share semantics are
      now documented prominently on `material.md`. A genuine
      deep-clone (allocate fresh `Shader`, copy every uniform value)
      remains a v0.12 task because the lazy-loaded textures make
      copy-on-write tricky to get right under concurrent renders.
      The user-side workaround today: build a fresh
      `Material::pbr()` and re-set the factors / textures.
- [ ] **`docs/api/scene/model/visible.md` See also** — already has
      pointers to `Model::set_visible` and the parent `Model` page.
      Verify alignment with sibling getter pages
      (`camera/position.md`, etc.) is consistent. Quick audit pass.

## Defer — v0.12 candidates

- [ ] **Light cap removal via storage-buffer path.** Move the
      shader-side `array<Light, 32>` (UBO) to `var<storage, read>
      lights: array<Light>` on backends that support storage
      buffers; keep the UBO path as the WebGL2 fallback. Removes the
      cap entirely for the forward path at O(N) per-fragment cost.
      The bumped 32-cap covers most scenes today, but a hundreds-of-
      lights scene wants this.
- [ ] **Clustered / Forward+ shading.** Tile / froxel-cull lights in
      a compute pass, fragment shader only iterates lights affecting
      its tile. Real "many lights" path — opt-in via a separate
      `Material::pbr_clustered()` constructor or similar so the
      simple forward path stays simple.
- [ ] **`Pass::scene_objects: Vec<Box<dyn SceneObject>>` with
      `apply_to_shader` replay machinery.** Two concrete impls
      (Camera, Light) behind a default-no-op trait. Consider direct
      `Weak<CameraObject>` / `Weak<LightObject>` fields on
      `PassObject` if a third impl never materialises. Today the
      trait is the right shape for composability; revisit when a
      third SceneObject arrives.
- [ ] **Light cap is per-Shader, not per-Pass.** Two Passes sharing
      a Material can each hold 32 lights → 64 total via the same
      shader. Whether that's a bug or a feature depends on intended
      semantics; pick one and document. (Acceptable today because
      lights bind to the shader's uniform slots; documenting the
      sharing behaviour is the right v0.11.2 move, the formal
      decision can wait.)
- [ ] **glTF main-thread blocking.** `gltf::import_slice` decodes
      images synchronously. Documented in `Scene::load.md` today
      with the worker-thread workaround; a future async API
      (`Scene::load_async`) could fan the decode work onto a thread
      pool / Web Worker.
