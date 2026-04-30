# Roadmap

Lean checklist organized by version. Each section copy-pastes into `CHANGELOG.md` as it ships.

**Versioning policy.** The minor version is defined by a theme milestone (0.11 = iOS + Android). API additions alone do not bump the minor — they land as patches on the current minor. Breaking changes bump the minor before 1.0.

**Current priorities.** Expanding examples and increasing technical capability / stability. Near-term target after that is a REPL with day-one hot reload (0.12).

---

## 0.11.1 — Examples: single-pass wave

- [ ] Hello ShaderToy Clone
- [ ] Hello custom geometry (vertex input)
- [ ] Hello Instances (simple)
- [ ] Big Particle System (stress test)

## 0.11.2 — Examples: multi-pass wave

- [ ] Simple multi-pass projected shadows
- [ ] Hello ExternalTextures (video processing)
- [ ] Hello multiple screens
- [ ] Hello screen regions / viewport
- [ ] Let's build a simple image editor
- [ ] Hello Shader Organ — per-frame post-process stack mutation (base FBM render + keyboard-toggled stack of warp/chromatic/blur/pixelate/invert/vignette/feedback-decay passes; exercises per-frame `Vec<Pass>` mutation and `TextureTarget` reuse)

## 0.11.3 — Stability: device-lost recovery

- [ ] Centralized device-lost handler: re-request adapter/device, rebuild render context, reconfigure surfaces/targets, clear or rebuild caches/pools safely
- [ ] Docs covering which state is preserved vs reinitialized
- [ ] Simulated-device-lost recovery test (where GPU backend permits)

## 0.11.4 — Stability: staging belt + descriptor caches

- [ ] Staging-belt style uploads for uniforms (adopt `wgpu::util::StagingBelt` or adapt `BufferPool` to per-frame belt flush; honor uniform alignment + backend constraints)
- [ ] Sampler cache keyed by sampler descriptor (LRU cap)
- [ ] Texture view cache keyed by view descriptor (LRU cap)
- [ ] No behavior change until consumed by later features; tests only

## 0.11.5 — Perf: pipeline warming + frame telemetry

- [ ] Async pipeline warming — background precompile per `(ShaderHash, format, sample_count)` when shaders/passes are registered; feature-gated
- [ ] Surface frame acquire telemetry — throttled counters / log-once warnings distinguishing target-local vs centralized retries on `Lost` / `Outdated`

## 0.11.6 — Stability: Apple compute-boundary auto-split (extended)

- [ ] Extend the existing `compute → compute` auto-split (shipped 0.11.0) to also cover the `compute (storage-write) → render (sampled-read)` hazard in a single command buffer (TBDR flush)

## 0.11.7 — Visual snapshot testing: coverage + harness

- [ ] Snapshot: circle fragment shader (deterministic params)
- [ ] Snapshot: compute_texture (deterministic inputs)
- [ ] Snapshot: particles_splat (seeded RNG; small tolerance or invariant assertions)
- [ ] Snapshot: storage texture pipelines (clear + splat sequence)
- [ ] Snapshot: mesh rendering with vertex + instance attributes
- [ ] Snapshot: MSAA rendering path and resolve
- [ ] Snapshot: push-constant native and uniform-fallback modes
- [ ] Harness: helper to snapshot a multi-pass render (`Vec<Pass>`) directly
- [ ] Harness: document `UPDATE_EXPECT` flow in CONTRIBUTING
- [ ] CI: macOS step to run snapshots and upload artifacts on failure
- [ ] Optional: headless flags on examples for snapshot integration tests

## 0.11.8 — Mobile: iOS example app

- [ ] `platforms/swift/examples/` xcodeproj consuming the SPM package
- [ ] Expand iOS healthchecks beyond headless smoke test: textures, immediates, frames

## 0.11.9 — Mobile: Android example app

- [ ] `platforms/kotlin/examples/` gradle project consuming the AAR
- [ ] Expand Android healthchecks beyond headless smoke test: textures, immediates, frames

## 0.11.10 — Mobile: publish packages

- [ ] Publish Kotlin AAR to Maven Central (Sonatype OSSRH credentials + GPG signing)
- [ ] Publish Swift Package to the Swift Package Index (register repo after first tag)

## 0.11.11 — API: surface helper + RenderPass revamp

- [ ] Core helper `create_target_from_surface(surface, size)` — removes duplication across Web/Python/iOS/Android
- [ ] Revamp `RenderPass` API — expose `wgpu::RenderPass` customizations with sensible defaults

## 0.11.12 — API: specialized aliases + custom blending

- [ ] `Compute` newtype for `Shader` (compute-only; `Shader` continues to allow both)
- [ ] `RenderPass` newtype for `Pass` (render-only)
- [ ] `ComputePass` newtype for `Pass` (compute-only)
- [ ] Custom blending

## 0.11.13 — API: JSON save/load

- [ ] `Mesh::load_*` helpers and JSON inputs
- [ ] Pass / pipeline setup save & load from JSON
- [ ] Shader state save & load as JSON (uniform values, textures) + JSON schema for default uniform values
- [ ] Re-add optional JSON shader source (feature-flagged; was removed in 0.11)

## 0.11.14 — Python: window integrations beyond RenderCanvas

- [ ] Qt
- [ ] WxWidgets
- [ ] glTF
- [ ] Jupyter

## 0.11.15 — Upstream: uniffi struct-rename support

- [ ] Contribute struct-rename support to uniffi upstream (only if needed for naming parity across bindings)

---

## 0.12.0 — REPL core: hot reload

Prototyping is one of the reasons this library exists; hot reload is day-one UX, not polish.

- [ ] `Shader::new_watched("./scene.wgsl")` on desktop — re-parses and re-compiles on file change; preserves last-known-good on compile error (no canvas teardown)
- [ ] `Shader::update_source(new_source)` — fallible swap that preserves previous on error
- [ ] `ShaderHealth { Ok | Stale { error } }` observable so UIs can surface diagnostics
- [ ] Uniform-state preservation across reloads: matching names keep values; new uniforms take defaults; removed uniforms drop silently

## 0.12.1 — REPL web: Vite HMR

- [ ] Vite plugin (or documented convention) that forwards `.wgsl` file changes to `shader.update_source()` on the JS side
- [ ] Parity with desktop: canvas keeps rendering last-good on compile error

## 0.12.2 — REPL shell

- [ ] Split editor/canvas, resizable divider, pane swap, fullscreen canvas toggle
- [ ] Monaco (or CodeMirror 6) with WGSL syntax highlighting
- [ ] Auto-apply on keystroke with 250 ms debounce
- [ ] Inline error highlighting: underline offending line, tooltip with naga's message

## 0.12.3 — Auto-generated uniform UI

- [ ] `Renderer::panel(&shader)` — auto-generated debug UI
- [ ] Widget rules per uniform type: slider (f32), stepper (i32/u32), checkbox (bool), XY pad (vec2), three sliders or color picker (vec3), four sliders or RGBA picker (vec4), expandable grid (matrices), file picker (textures)
- [ ] WGSL comment annotations: `// @range a..b @step s @default d`, `// @color`, `// @xy`, `// @file`
- [ ] Backend-agnostic rendering: egui on desktop, HTML form on web (mobile native is stretch)
- [ ] Writes flow through `shader.set(...)`

## 0.12.4 — REPL sharing

- [ ] Shareable URLs: encode `{ source, uniforms, inputs }` as gzip + base64 in URL hash; decoding re-hydrates full REPL state
- [ ] "Save to gist" button (anonymous)

## 0.12.5 — Showcase Gallery

- [ ] `/gallery` route with 20–30 hand-picked demos: glTF viewer, audio-reactive music viz, compute simulation (fluid or boids or slime mold), SDF raymarcher, post-processing stack demo, Shadertoy port, "build a shader in 10 lines" teaser
- [ ] Each card: thumbnail, description, "Open in REPL", "View source"
- [ ] "Shader of the Day" rotation on homepage
- [ ] Example: Hello Live-Coding (REPL walkthrough)

## 0.12.6 — External textures: cross-platform implementation

The `Renderer::create_external_texture` API surface was renamed and made
cross-platform in 0.11.0 (every binding exposes the entry point), but every
implementation is a stub that returns `RendererError::Error("not implemented yet")`.
This slot fills in the per-platform plumbing so video frames can be sampled
zero-copy in WGSL via `texture_external` / `textureSampleBaseClampToEdge`.

- [ ] Web: real `HTMLVideoElement` → `wgpu::ExternalTexture` mapping (wgpu-web
      already exposes the import hooks; the stub at
      [src/renderer/external_texture.rs](src/renderer/external_texture.rs)
      `create_external_texture` is where the impl lands)
- [ ] iOS: `CVPixelBuffer` → `wgpu::ExternalTexture` via
      `CVMetalTextureCacheCreateTextureFromImage`. Swift extension wraps the
      uniffi `createExternalTexture(sourcePtr: UInt64)` shim around a real
      `CVPixelBuffer` argument so callers don't see the raw pointer.
- [ ] Android: `SurfaceTexture` → `EGLImage` → `OES_EGL_image_external` sampled
      texture. Kotlin extension wraps the uniffi `createExternalTexture(ptr: ULong)`
      shim around a real `SurfaceTexture` argument.
- [ ] Python: decide whether to expose at all (Python lacks a portable native
      video-frame source — current docs steer users to `Texture.write()`); if
      adding, accept a `numpy.ndarray` or raw bytes + format/size.
- [ ] Move the public doc back to `docs/api/core/renderer/create_external_texture.md`
      (currently parked at `docs/api/core/renderer/hidden/` while unimplemented)
      and re-flow the per-language `_js.md` / `_py.md` / `_swift.md` / `_kotlin.md`
      overrides into idiomatic per-platform examples.
- [ ] Healthcheck coverage: smoke test that decodes one frame on each platform
      and reads back via `Renderer::read_texture`.

---

## 0.13.0 — Composition: preprocessor + `use` statements

Function-level WGSL composition. No registry yet — this ships the preprocessor and local resolution only.

- [ ] Preprocessor pipeline: tokenize minimally → hoist `use` imports → walk deps (DAG, cycle detection) → tree-shake → mangle names → emit
- [ ] Form A syntax: `use @fc/sdf/sphere;`, `use @fc/sdf/smooth_min as smin;` at top of WGSL file
- [ ] Transitive deps: imported functions may themselves `use` other slugs
- [ ] Tree-shaking: only emit functions reachable from user entry points; unused imports are warnings, not errors
- [ ] Name mangling: imported functions prefixed (`__fc_sdf_sphere`); user-facing names are thin alias shims
- [ ] Function-pack invariants (enforced by preprocessor): no `@group`/`@binding`, no entry points, exactly one public `fn`, `const` allowed at module scope, no `var<private>`/`var<workgroup>`
- [ ] Errors: `ShaderError::{ImportCycle, NameCollision, SignatureMismatch, InvalidRegistryEntry}`

## 0.13.1 — Composition: inline slug / URL form

- [ ] Form B syntax: `@fc/sdf/sphere(p, 1.0)` anywhere in a WGSL body — slug/URL immediately followed by `(`
- [ ] Preprocessor rewrites each unique inline slug to a mangled call and hoists a synthetic `use` for resolution
- [ ] Full `https://…(...)` URLs accepted the same way
- [ ] `use … as <alias>` takes precedence when a name also appears inline

## 0.13.2 — Composition: registry client

- [ ] `Shader::new("@fc/bloom")` / `Shader::new("/bloom")` / `Shader::new("@alice/neon-grid")` resolution
- [ ] Version pinning: `@fc/bloom@1.2`, `@fc/bloom#sha256:abc…`
- [ ] Full-URL pinning: `https://shaders.fragmentcolor.org/.../bloom.wgsl#sha256:abc…`
- [ ] Local cache (platform data dir native, IndexedDB web); keyed by sha256 of manifest+body; default 256 MiB LRU cap
- [ ] `Renderer::new_with_resolve_policy(Latest | Locked | Offline)` — `Locked` refuses unpinned slugs at build time; `Offline` serves only from cache
- [ ] `ShaderError::ResolveError { slug, cause }` for network / 404 / hash mismatch

## 0.13.3 — Composition: registry service + CLI

- [ ] Hosted registry at `shaders.fragmentcolor.org` (static origin, immutable content-addressed entries)
- [ ] Registry entry model: TOML manifest + WGSL body, declared function signature, transitive deps
- [ ] Two entry kinds: function-pack (primary) and pass-shader (pipeline stages)
- [ ] `fragmentcolor publish [path]` — validate manifest + body, compute sha256, POST with signing key
- [ ] `fragmentcolor resolve <slug>` — print resolved URL + hash + signature
- [ ] `fragmentcolor cache --list` / `cache --clear`
- [ ] `fragmentcolor lint <file.wgsl>` — resolve imports, run naga, print errors without rendering

## 0.13.4 — Composition: pipeline-level

- [ ] `compose(["scene", "/bloom", "/tonemap"])` → `Vec<Pass>` where each stage samples the previous pass's output
- [ ] Chainable builder: `shader.then("bloom").then("tonemap")`
- [ ] Mixin-level composition (merging multiple pass-shaders into one stage) is deferred (see Unversioned backlog)

## 0.13.5 — WGSL standard library rollout

- [ ] `@fc/sdf/*` — sphere, box, torus, capsule, plane, union, intersect, subtract, smooth_min, smooth_max
- [ ] `@fc/hash/*` — pcg2d, wang, hash11, hash21, hash31
- [ ] `@fc/noise/*` — value2, fbm2, simplex2, simplex3, worley2, gradient2
- [ ] `@fc/color/*` — srgb_to_linear, linear_to_srgb, hsl_to_rgb, rgb_to_hsl, oklab_to_linear_srgb, kelvin_to_rgb, gamma
- [ ] `@fc/map/*` — rotate2, twirl, barrel, fisheye, kaleidoscope
- [ ] `@fc/camera/*` — perspective, orthographic, look_at, screen_to_world
- [ ] `@fc/lighting/*` — lambert, blinn_phong, ggx, fresnel_schlick, cook_torrance
- [ ] `@fc/easing/*` — in_out_cubic, elastic_out, bounce_out, back_in_out
- [ ] `@fc/gradient/*` — palette_iq, turbo, viridis, magma
- [ ] `@fc/sample/*` — halton, blue_noise, hemisphere_cosine
- [ ] Example: Hello Composition

---

## 0.14.0 — Magic uniforms

- [ ] Auto-populated recognized names: `time`, `delta_time`, `frame`, `resolution`, `mouse` (xy + click state), `scroll`, `aspect`
- [ ] Auto-bind policy: only populate if the shader declares a matching-type uniform; never overwrite an explicit `shader.set()`
- [ ] Same convention across Rust, Web, Python, Swift, Kotlin

## 0.14.1 — Shadertoy bridge

- [ ] `Shader::from_shadertoy(url)` — fetch public Shadertoy, translate GLSL→WGSL via naga, wrap the `mainImage(out vec4, in vec2)` contract
- [ ] `Shader::toy(source)` — accept pasted bodies
- [ ] Auto-map `iTime`, `iResolution`, `iMouse`, `iFrame`, `iChannel0..3` → magic uniforms + texture bindings
- [ ] Multi-buffer Shadertoy layouts (Buffer A/B/C/D) → multi-pass `Vec<Pass>` automatically
- [ ] Registry shorthand: `@toy/abcdef` → Shadertoy ID
- [ ] Example: Hello ShaderToy

## 0.14.2 — Audio-reactive pipeline

- [ ] `renderer.create_audio_texture(source)` — FFT bins + waveform as `texture_1d<f32>`
- [ ] Sources: `<audio>` element, `AudioContext` node, microphone, URL, file (native)
- [ ] Configurable sample rate + FFT size; defaults suitable for music viz
- [ ] Example: Hello Audio-Reactive (rings / spectrum visualization)

## 0.14.3 — Live input uniforms + textures

- [ ] `renderer.create_webcam_texture()` — live camera feed as `texture_2d<f32>` (web: `getUserMedia`; desktop: platform APIs; mobile later)
- [ ] Unify `create_video_texture(path_or_url)` across web and desktop
- [ ] Unified input uniforms (build on 0.14.0): `mouse`, `keyboard` (bitset), `gamepad` (stick + buttons), `touches` (array of xy)
- [ ] Example: Hello Webcam (live video post-processing)

---

## 0.15.0 — Model loading: glTF / glb

- [ ] `Mesh::from_gltf("model.glb")` — positions, normals, UVs, tangents, vertex colors, skin weights
- [ ] `Mesh::from_gltf_all(...)` → `Vec<Mesh>` for multi-primitive models
- [ ] Skinned animation: joint matrices uploaded as a storage buffer; vertex shader samples per-vertex

## 0.15.1 — PBR material registry + glTF integration

- [ ] `@fc/pbr` registry template with PBR metallic-roughness
- [ ] Auto-bind baseColor / normal / metallic-roughness / emissive textures from glTF materials
- [ ] Example: Hello glTF (model viewer with auto-camera + PBR)

## 0.15.2 — Post-processing stack templates

- [ ] `postfx([Bloom::default(), ToneMap::aces(), Vignette::subtle()])` → `Vec<Pass>`
- [ ] Registry stages: `@fc/bloom`, `@fc/ssao`, `@fc/tonemap-aces`, `@fc/tonemap-reinhard`, `@fc/chromatic`, `@fc/vignette`, `@fc/grade`, `@fc/film-grain`, `@fc/motion-blur`, `@fc/fxaa`, `@fc/depth-of-field`
- [ ] Typed parameters per stage; custom stages implement a trait

---

## 0.16.0 — Visual snapshot testing (formalized)

- [ ] `Renderer::render_to_hash(&shader) -> Sha256`
- [ ] PNG-diff with tolerance for cross-GPU determinism
- [ ] CI integration with artifact upload on diff
- [ ] Document `UPDATE_EXPECT=1` rebaselining flow

## 0.16.1 — Automatic shader minification

- [ ] Build-time step: strip comments, rename identifiers in bundled + registry shaders
- [ ] Per-platform specialization (native: push-constants; web: uniform rewrite) as a build step rather than runtime

---

## 0.17.0 — Backend expansion

- [ ] WebGL2 downlevel for fragment-only single-pass shaders; detect at init, fall back gracefully
- [ ] Canvas2D fallback for `texture_external` video filters (the "always something renders" promise)
- [ ] `Renderer::capabilities()` — feature-detection API (compute, storage textures, push constants, …)

---

## 0.18.0 — Community shaders

- [ ] Registry-backed submissions with review
- [ ] Fork/remix flow: open any registry shader in the REPL, edit, re-publish under your namespace
- [ ] Tags, search, trending
- [ ] Per-shader stats (views, forks, stars)
- [ ] Moderation pipeline (infinite loops, malicious resource usage)

---

## 1.0.0 — Stable release

- [ ] Lock public API surface; document stability guarantees
- [ ] LLM copilot support: `llms.txt` + [MCP](https://modelcontextprotocol.io/introduction) server so assistants author correct FragmentColor code without hallucinating
- [ ] Website + docs polish pass
- [ ] Internationalization groundwork for docs

---

## Post-1.0.0 — `fragmentcolor-kit` (separate package)

Higher-level framework, shipped separately (analogous to SvelteKit / Next.js). Only if the community asks; the core library stands on its own.

- [ ] `Scene` with transform hierarchy; cameras (perspective / orthographic / cubemap); lights
- [ ] Material presets: PBR, Toon, Unlit, Glass
- [ ] Scene-level features: frustum culling, sorting, LOD, shadow mapping
- [ ] GPU-driven particle system (compute)
- [ ] Rapier physics integration (optional)
- [ ] Declarative component wrappers: React, Svelte, Vue (one thin package each)
- [ ] Scene serialization to JSON (round-trip)
- [ ] 2D UI / HUD overlay layer above 3D
- [ ] Gizmos for in-editor manipulation
- [ ] Example: Scene tree + camera tutorial
- [ ] Moonshot: a Logo-like programming language for procedural shaders

**Not planned**: ECS, plugin framework, custom DSL. Trait-based extension points already cover the real cases.

---

## Unversioned backlog

Rough ideas without a concrete target yet. Promote to a version when active.

- [ ] MSAA edge cases (`resolve_target` in `RenderPassColorAttachments` — beyond current MSAA)
- [ ] Composition mixin-level: merging multiple pass-shaders into one pipeline stage (binding-conflict policy, `use … remap(0, 1)` syntax)
- [ ] Public preprocessor API: `Shader::preprocess(source) -> String` for inspection / shipping pre-resolved shaders
- [ ] Registry: signed uploads to deter namespace squatting
- [ ] Halve conversion traits with `Borrow<B>` across `Into<UniformData>` and `Into<ColorTarget>` paths
