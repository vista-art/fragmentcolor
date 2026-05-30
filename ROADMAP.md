# Roadmap

Themed checklist organized by minor version. Within a minor version, work
ships in patch releases as it lands; the roadmap doesn't try to predict patch
order — only what's in scope for the milestone.

**Versioning policy.** The minor version is defined by a theme milestone (0.11
= iOS + Android). API additions alone do not bump the minor — they land as
patches on the current minor. Breaking changes bump the minor before 1.0.

**Current priorities.** Wrapping up 0.11.x mobile-launch stability and content,
then 0.12.x asset pipeline (KTX2 + streaming + glTF) — the production-content
cut. Live media (0.13.x) and the live-coding REPL with WGSL composition
(0.14.x) follow.

---

## 0.11.x — Mobile launch follow-through

Stability and polish for the recent iOS/Android release. Tighten the existing
surface, expand examples and tutorials, seed a small shader collection that
demonstrates the registry mechanism shipped in the 0.11.0 shader-composition
work.

### 0.11.1 — pre-shipping unblockers
- [ ] **`external_texture` minimum-viable example.** The cross-platform API
      surface was renamed in 0.11.0 but every implementation is still a stub
      returning `RendererError::Error("not implemented yet")`. We can't
      ship the public-facing `docs/api/web/external_texture.md` until at
      least one concrete runtime example works end-to-end (web's
      `HTMLVideoElement` → `wgpu::ExternalTexture` path is the obvious
      first; full per-platform plumbing is tracked in 0.13.x). Until that
      example exists the doc stays parked at `docs/api/web/hidden/`.

### Stability + perf
- [ ] Centralized device-lost handler: re-request adapter/device, rebuild render context, reconfigure surfaces/targets, clear or rebuild caches/pools safely; docs covering preserved-vs-reinitialized state; simulated-device-lost test where the GPU backend permits
- [ ] Extend the existing `compute → compute` auto-split (shipped 0.11.0) to also cover the `compute (storage-write) → render (sampled-read)` hazard in a single command buffer (Apple TBDR flush)
- [ ] Staging-belt style uploads for uniforms (adopt `wgpu::util::StagingBelt` or adapt `BufferPool` to per-frame belt flush; honor uniform alignment + backend constraints)
- [ ] Sampler cache + texture-view cache keyed by descriptor (LRU caps); no behavior change until later features consume them
- [ ] Async pipeline warming — background precompile per `(ShaderHash, format, sample_count)` when shaders/passes register; feature-gated
- [ ] Surface frame-acquire telemetry — throttled counters / log-once warnings distinguishing target-local vs centralized retries on `Lost` / `Outdated`

### Examples + tutorials
- [ ] Hello ShaderToy Clone
- [ ] Hello custom geometry (vertex input)
- [ ] Hello Instances (simple)
- [ ] Big Particle System (stress test)
- [ ] Simple multi-pass projected shadows
- [ ] Hello multiple screens
- [ ] Hello screen regions / viewport
- [ ] Let's build a simple image editor
- [ ] Hello Shader Organ — per-frame post-process stack mutation (base FBM render + keyboard-toggled stack of warp/chromatic/blur/pixelate/invert/vignette/feedback-decay passes; exercises per-frame `Vec<Pass>` mutation and `TextureTarget` reuse)
- [ ] Step-by-step tutorials in `docs/api/*` (multi-block format; the goal is full tutorials, not just hover-doc fodder)

### Snapshot testing
- [ ] `Renderer::render_to_hash(&shader) -> Sha256`
- [ ] PNG-diff with tolerance for cross-GPU determinism
- [ ] CI integration with artifact upload on diff
- [ ] Coverage: circle fragment shader, compute_texture, particles_splat (seeded RNG), storage texture pipelines, mesh with vertex+instance attributes, MSAA + resolve, push-constant native + uniform-fallback modes
- [ ] Harness: helper to snapshot a multi-pass render (`Vec<Pass>`) directly
- [ ] Document `UPDATE_EXPECT=1` rebaselining flow in CONTRIBUTING; macOS CI step
- [ ] Optional: headless flags on examples for snapshot integration tests

### Mobile
- [ ] iOS example app: `platforms/swift/examples/` xcodeproj consuming the SPM package
- [ ] Android example app: `platforms/kotlin/examples/` gradle project consuming the AAR
- [ ] Expand iOS + Android healthchecks beyond headless smoke tests: textures, immediates, frames
- [ ] Publish Kotlin AAR to Maven Central (Sonatype OSSRH credentials + GPG signing)
- [ ] Publish Swift Package to the Swift Package Index (register repo after first tag)

### API refinement
- [ ] Core helper `create_target_from_surface(surface, size)` — removes duplication across Web / Python / iOS / Android
- [ ] Revamp `RenderPass` API — expose `wgpu::RenderPass` customizations with sensible defaults
- [ ] Specialized aliases: `Compute` newtype for `Shader` (compute-only); `RenderPass` newtype for `Pass` (render-only); `ComputePass` newtype for `Pass` (compute-only)
- [ ] Custom blending
- [ ] JSON save/load: `Mesh::load_*` helpers + JSON inputs; pass / pipeline setup save & load; shader state save & load (uniform values, textures) + JSON schema for default uniform values; re-add optional JSON shader source (feature-flagged; was removed in 0.11)

### Python window integrations
- [ ] Qt
- [ ] WxWidgets
- [ ] Jupyter

### Shader collection
- [ ] Curate ~10–20 WGSL shaders into the registry mechanism (sdf primitives, hash/noise, color helpers); seed `shaders.fragmentcolor.org` content for the registry that ships in 0.14.x

### Upstream
- [ ] Contribute struct-rename support to uniffi upstream (only if needed for naming parity across bindings)

---

## 0.12.x — Asset pipeline: textures + 3D models

KTX2 container support with compressed texture formats and progressive
streaming; glTF/PBR model loading with skinning; post-processing stack
templates. The production-content cut — anything that ships textures or
geometry from a real authoring pipeline.

### KTX2 container + compressed texture formats

Initial KTX2 support shipped as 0.11.x patches (see CHANGELOG): parse-only via
the `ktx2` crate, `TextureInput::{Ktx2Bytes, Ktx2Path, Ktx2Url}` variants,
opportunistic compression-feature requests at device creation, dispatch for
RGBA8 / RGBA16F / R16 / Rg16 / Bgra8 / BC1-7 / ASTC 4×4 + 8×8 / ETC2,
pre-baked mip chain uploaded verbatim, capability-aware errors when the
active GPU lacks a required compression feature, synthetic in-memory KTX2
round-trip test. The remaining work below covers the gaps consumers will
hit as scope grows.

- [ ] Magic-number sniff in `TextureInput::Bytes` and `.ktx2` extension recognition in `TextureInput::Path` so existing `Renderer::create_texture(path)` auto-routes to the KTX2 loader without callers picking the explicit `Ktx2Path` / `Ktx2Bytes` variant
- [ ] Cube map support (KTX2 `faceCount = 6`): upload all 6 faces into one texture; expose a `TextureViewDimension::Cube` view (currently rejected with a clear error)
- [ ] Array texture support (KTX2 `layerCount > 1`): `TextureViewDimension::D2Array` (currently rejected)
- [ ] 3D texture support (KTX2 `pixelDepth > 1`): `TextureViewDimension::D3` (currently rejected)
- [ ] `TextureOptions.dimension: Option<TextureViewDimension>` so callers can request `Cube` / `D2Array` / `D3` views explicitly; auto-pick from container metadata when `None`
- [ ] Decompress KTX2 supercompression: `SupercompressionScheme::Zstandard` and `::ZLIB`. BasisLZ is covered in the Basis Universal section below; loads of supercompressed payloads currently fail with a clear error pointing at this gap
- [ ] Public `TextureFormat` enum gains the BC family (`Bc1RgbaUnorm/Srgb`, `Bc2RgbaUnorm/Srgb`, `Bc3RgbaUnorm/Srgb`, `Bc4RUnorm/Snorm`, `Bc5RgUnorm/Snorm`, `Bc6hRgbUfloat/Float`, `Bc7RgbaUnorm/Srgb`) so callers can request these via `TextureOptions.format`. The KTX2 path uses `wgpu::TextureFormat` directly today, bypassing the public enum
- [ ] Public `TextureFormat` enum gains ASTC variants (4×4 through 12×12, Unorm + UnormSrgb + HDR), ETC2 (`Etc2Rgb8Unorm/Srgb`, `Etc2Rgb8A1Unorm/Srgb`, `Etc2Rgba8Unorm/Srgb`), and EAC (`EacR11Unorm/Snorm`, `EacRg11Unorm/Snorm`)
- [ ] `bytes_per_pixel` becomes `bytes_per_block(format) -> (u32, BlockSize)` — the unit of upload for compressed formats is the 4×4 (or larger) block, not the pixel; current name is misleading
- [ ] Per-platform CI fixtures + tests: pre-baked BC7 on desktop, ASTC 4×4 on Apple, ETC2 on Android emulator; cube-map smoke test (6-face); failure-mode test (BC7 loaded on a no-BC adapter returns the capability error, not silent garbage)
- [ ] Per-platform availability matrix in `docs/guides/loading-compressed-textures.md` (desktop / iOS / Android / WebGPU)
- [ ] WebGL2 compressed-texture coverage explicitly tracked as a non-goal — Basis Universal handles it instead

### Progressive texture streaming
KTX2 stores levels in reverse order (smallest mip last), which is perfect for
sequential loads: parse the header, fetch the smallest level, render at low-res
immediately, fetch larger levels in the background and bump the sampler's
`lod_min_clamp` down as each one arrives.

- [ ] New public type `StreamingTexture` wrapping `Texture` plus loading state (`Loading { available, total } | Complete`); `Deref<Target = Texture>` so it drops in anywhere a `Texture` works
- [ ] `Renderer::stream_texture(input) -> StreamingTexture` — synchronous preface fetches the KTX2 header + smallest mip, then returns; rendering proceeds with `lod_min_clamp` set to whatever's loaded
- [ ] Background mip arrival pumps the texture: each new level uploaded via `queue.write_texture` at the right `mip_level`; `lod_min_clamp` decremented; no re-creation, no bind-group churn
- [ ] `StreamingTexture::on_progress(impl FnMut(StreamingState) + Send + 'static)` hook for UIs (progress bars, "loading…" overlays); `Drop` cancels in-flight fetches
- [ ] Web fetch path: `fetch(url, { headers: { Range: "bytes=N-M" } })` with bounded concurrency
- [ ] Native + mobile fetch path: extend `crate::net::fetch_bytes` with a Range variant (`ureq` already supports the header); one worker thread per active stream; backpressure when budget exhausted
- [ ] `Renderer::set_streaming_budget(parallel: u32, bytes_per_sec: Option<u64>)` with sensible defaults; `bytes_per_sec` rate-limits the fetcher (useful on cellular)
- [ ] Tests: synthetic 5-level KTX2 fixture served from a `tiny_http` temp server (dev-dep); assert mips arrive smallest-first; assert drop cancels pending fetches; assert `lod_min_clamp` decreases monotonically
- [ ] Docs: `docs/guides/streaming-textures.md` covering the LOD-clamp story; example "Hello Streaming Textures" loading a 4K KTX2

### Basis Universal transcoding (optional)
Basis Universal is the "author once, deploy everywhere" supercompression
layer. A single UASTC- or ETC1S-encoded KTX2 file transcodes at load time into
the target device's native compressed format (BC7 / ASTC / ETC2). Solves the
shipping problem the compressed-format work above exposes — without Basis,
callers have to ship per-platform asset bundles. Heavy enough (C++ dep, ~5 MiB
compiled) that it ships behind a feature flag.

- [ ] New crate feature `basis` enabling `basis-universal` (which wraps Khronos's official C++ library via `basis-universal-sys`); opt-in only
- [ ] Detect Basis-encoded payloads in KTX2 (`supercompressionScheme = BasisLZ` or UASTC) and route through the transcoder instead of uploading raw
- [ ] Adapter capability detection picks the transcode target: desktop with BC → `Bc7RgbaUnorm` (UASTC) / `Bc1RgbaUnorm` + `Bc3RgbaUnorm` (ETC1S); Apple with ASTC → `Astc4x4Unorm` (both); Android with ETC2 → `Etc2Rgba8Unorm` (both); universal fallback → `Rgba8Unorm` (slowest, ~4× memory, only when no compressed format is available)
- [ ] Compatible with streaming above: each mip transcoded on arrival rather than waiting for the full chain
- [ ] When the `basis` feature is off, Basis-encoded files fail with `TextureError::BasisFeatureDisabled` whose message points at the feature flag — no silent uncompressed fallback (would be surprising and slow)
- [ ] Healthcheck: same Basis-encoded `.ktx2` fixture uploaded on every CI matrix entry; readback compares against per-platform tolerance (lossy compression, exact-equal isn't realistic)
- [ ] Docs: `docs/guides/basis-universal-workflow.md` covering the asset pipeline (`basisu` CLI to author files; fragmentcolor with `--features basis` to consume them)

### glTF / glb model loading
- [ ] `Mesh::from_gltf("model.glb")` — positions, normals, UVs, tangents, vertex colors, skin weights
- [ ] `Mesh::from_gltf_all(...) -> Vec<Mesh>` for multi-primitive models
- [ ] Skinned animation: joint matrices uploaded as a storage buffer; vertex shader samples per-vertex
- [ ] PBR material registry (`@fc/pbr` template with metallic-roughness)
- [ ] Auto-bind glTF materials: baseColor / normal / metallic-roughness / emissive textures wired up automatically
- [ ] Other formats considered as the ecosystem demands (USD/USDZ for Apple workflows, OBJ for legacy assets); glTF first since it's the de-facto modern interchange
- [ ] Example: Hello glTF (model viewer with auto-camera + PBR)

### Post-processing stack templates
- [ ] `postfx([Bloom::default(), ToneMap::aces(), Vignette::subtle()]) -> Vec<Pass>`
- [ ] Registry stages: `@fc/bloom`, `@fc/ssao`, `@fc/tonemap-aces`, `@fc/tonemap-reinhard`, `@fc/chromatic`, `@fc/vignette`, `@fc/grade`, `@fc/film-grain`, `@fc/motion-blur`, `@fc/fxaa`, `@fc/depth-of-field`
- [ ] Typed parameters per stage; custom stages implement a trait
- [ ] Example: Hello Post-FX

---

## 0.13.x — Live media + interactive inputs

External textures (video), audio-reactive textures, magic uniforms, live input
sources (webcam, microphone, gamepad, touch). The interactive-content cut —
what creative coders need for music videos, game prototypes, AR/VR demos.

### External textures (video)
The `Renderer::create_external_texture` API surface was renamed and made
cross-platform in 0.11.0, but every implementation is still a stub that
returns `RendererError::Error("not implemented yet")`. This slot fills in
per-platform plumbing so video frames sample zero-copy in WGSL via
`texture_external` / `textureSampleBaseClampToEdge`.

- [ ] Web: real `HTMLVideoElement` → `wgpu::ExternalTexture` mapping (wgpu-web already exposes the import hooks; the stub at [src/renderer/external_texture.rs](src/renderer/external_texture.rs) is where the impl lands)
- [ ] iOS: `CVPixelBuffer` → `wgpu::ExternalTexture` via `CVMetalTextureCacheCreateTextureFromImage`; Swift extension wraps the uniffi shim around a real `CVPixelBuffer` argument so callers don't see the raw pointer
- [ ] Android: `SurfaceTexture` → `EGLImage` → `OES_EGL_image_external` sampled texture; Kotlin extension wraps the uniffi shim around a real `SurfaceTexture` argument
- [ ] Python: decide whether to expose at all (Python lacks a portable native video-frame source — current docs steer to `Texture.write()`); if adding, accept a `numpy.ndarray` or raw bytes + format/size
- [ ] Move the public doc back to `docs/api/core/renderer/create_external_texture.md` (currently parked at `docs/api/core/renderer/hidden/`); re-flow per-language `_js.md` / `_py.md` / `_swift.md` / `_kotlin.md` overrides into idiomatic per-platform examples
- [ ] Healthcheck: smoke test that decodes one frame on each platform and reads back via `Renderer::read_texture`

### Magic uniforms + Shadertoy bridge
- [ ] Auto-populated recognized names: `time`, `delta_time`, `frame`, `resolution`, `mouse` (xy + click state), `scroll`, `aspect`
- [ ] Auto-bind policy: only populate if the shader declares a matching-type uniform; never overwrite an explicit `shader.set()`
- [ ] Same convention across Rust, Web, Python, Swift, Kotlin
- [ ] `Shader::from_shadertoy(url)` — fetch public Shadertoy, translate GLSL→WGSL via naga, wrap the `mainImage(out vec4, in vec2)` contract
- [ ] `Shader::toy(source)` — accept pasted bodies
- [ ] Auto-map `iTime`, `iResolution`, `iMouse`, `iFrame`, `iChannel0..3` → magic uniforms + texture bindings
- [ ] Multi-buffer Shadertoy layouts (Buffer A/B/C/D) → multi-pass `Vec<Pass>` automatically
- [ ] Registry shorthand: `@toy/abcdef` → Shadertoy ID
- [ ] Example: Hello ShaderToy

### Audio-reactive
- [ ] `renderer.create_audio_texture(source)` — FFT bins + waveform as `texture_1d<f32>`
- [ ] Sources: `<audio>` element, `AudioContext` node, microphone, URL, file (native)
- [ ] Configurable sample rate + FFT size; defaults suitable for music viz
- [ ] Example: Hello Audio-Reactive (rings / spectrum visualization)

### Live input
- [ ] `renderer.create_webcam_texture()` — live camera feed as `texture_2d<f32>` (web: `getUserMedia`; desktop: platform APIs; mobile later)
- [ ] Unify `create_video_texture(path_or_url)` across web and desktop
- [ ] Unified input uniforms (build on Magic uniforms above): `mouse`, `keyboard` (bitset), `gamepad` (stick + buttons), `touches` (array of xy)
- [ ] Example: Hello Webcam (live video post-processing)

---

## 0.14.x — Live coding REPL + WGSL composition

Hot reload, browser REPL with auto-generated uniform UI, shader sharing, and
function-level WGSL composition with a hosted registry. The
developer-experience layer.

Prototyping is one of the reasons this library exists; hot reload is day-one
UX, not polish.

### Hot reload
- [ ] `Shader::new_watched("./scene.wgsl")` on desktop — re-parses and re-compiles on file change; preserves last-known-good on compile error (no canvas teardown)
- [ ] `Shader::update_source(new_source)` — fallible swap that preserves previous on error
- [ ] `ShaderHealth { Ok | Stale { error } }` observable so UIs can surface diagnostics
- [ ] Uniform-state preservation across reloads: matching names keep values; new uniforms take defaults; removed uniforms drop silently
- [ ] Vite plugin (or documented convention) that forwards `.wgsl` file changes to `shader.update_source()` on the JS side; canvas keeps rendering last-good on compile error

### REPL shell + auto UI
- [ ] Split editor/canvas, resizable divider, pane swap, fullscreen canvas toggle
- [ ] Monaco (or CodeMirror 6) with WGSL syntax highlighting
- [ ] Auto-apply on keystroke with 250 ms debounce
- [ ] Inline error highlighting: underline offending line, tooltip with naga's message
- [ ] `Renderer::panel(&shader)` — auto-generated debug UI
- [ ] Widget rules per uniform type: slider (f32), stepper (i32/u32), checkbox (bool), XY pad (vec2), three sliders or color picker (vec3), four sliders or RGBA picker (vec4), expandable grid (matrices), file picker (textures)
- [ ] WGSL comment annotations: `// @range a..b @step s @default d`, `// @color`, `// @xy`, `// @file`
- [ ] Backend-agnostic rendering: egui on desktop, HTML form on web (mobile native is stretch)
- [ ] Writes flow through `shader.set(...)`

### Sharing + gallery
- [ ] Shareable URLs: encode `{ source, uniforms, inputs }` as gzip + base64 in URL hash; decoding re-hydrates full REPL state
- [ ] "Save to gist" button (anonymous)
- [ ] `/gallery` route with 20–30 hand-picked demos: glTF viewer, audio-reactive music viz, compute simulation (fluid or boids or slime mold), SDF raymarcher, post-processing stack demo, Shadertoy port, "build a shader in 10 lines" teaser
- [ ] Each card: thumbnail, description, "Open in REPL", "View source"
- [ ] "Shader of the Day" rotation on homepage
- [ ] Example: Hello Live-Coding (REPL walkthrough)

### Composition: preprocessor
- [ ] Preprocessor pipeline: tokenize minimally → hoist `use` imports → walk deps (DAG, cycle detection) → tree-shake → mangle names → emit
- [ ] Form A syntax: `use @fc/sdf/sphere;`, `use @fc/sdf/smooth_min as smin;` at top of WGSL file
- [ ] Form B syntax: `@fc/sdf/sphere(p, 1.0)` anywhere in a WGSL body — slug/URL immediately followed by `(`; preprocessor rewrites to a mangled call and hoists a synthetic `use`; full `https://…(...)` URLs accepted the same way; `use … as <alias>` takes precedence when a name also appears inline
- [ ] Transitive deps: imported functions may themselves `use` other slugs
- [ ] Tree-shaking: only emit functions reachable from user entry points; unused imports are warnings, not errors
- [ ] Name mangling: imported functions prefixed (`__fc_sdf_sphere`); user-facing names are thin alias shims
- [ ] Function-pack invariants (enforced by preprocessor): no `@group`/`@binding`, no entry points, exactly one public `fn`, `const` allowed at module scope, no `var<private>` / `var<workgroup>`
- [ ] Errors: `ShaderError::{ImportCycle, NameCollision, SignatureMismatch, InvalidRegistryEntry}`

### Composition: registry client + service
- [ ] `Shader::new("@fc/bloom")` / `Shader::new("/bloom")` / `Shader::new("@alice/neon-grid")` resolution
- [ ] Version pinning: `@fc/bloom@1.2`, `@fc/bloom#sha256:abc…`; full-URL pinning works the same
- [ ] Local cache (platform data dir native, IndexedDB web); keyed by sha256 of manifest+body; default 256 MiB LRU cap
- [ ] `Renderer::new_with_resolve_policy(Latest | Locked | Offline)` — `Locked` refuses unpinned slugs at build time; `Offline` serves only from cache
- [ ] `ShaderError::ResolveError { slug, cause }` for network / 404 / hash mismatch
- [ ] Hosted registry at `shaders.fragmentcolor.org` (static origin, immutable content-addressed entries)
- [ ] Registry entry model: TOML manifest + WGSL body, declared function signature, transitive deps
- [ ] Two entry kinds: function-pack (primary) and pass-shader (pipeline stages)
- [ ] CLI: `fragmentcolor publish [path]`, `resolve <slug>`, `cache --list` / `--clear`, `lint <file.wgsl>`

### Composition: pipeline-level + WGSL stdlib rollout
- [ ] `compose(["scene", "/bloom", "/tonemap"]) -> Vec<Pass>` where each stage samples the previous pass's output
- [ ] Chainable builder: `shader.then("bloom").then("tonemap")`
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

### Build-time minification
- [ ] Build-time step: strip comments, rename identifiers in bundled + registry shaders
- [ ] Per-platform specialization (native: push-constants; web: uniform rewrite) as a build step rather than runtime

---

## 0.15.x — Backend expansion + community ecosystem

WebGL2 fallback for downlevel browsers, capability detection, community shader
gallery with submissions and remix flow.

### Backend expansion
- [ ] WebGL2 downlevel for fragment-only single-pass shaders; detect at init, fall back gracefully
- [ ] Canvas2D fallback for `texture_external` video filters (the "always something renders" promise)
- [ ] `Renderer::capabilities()` — feature-detection API (compute, storage textures, push constants, compressed format families, …)

### Community shaders
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

Higher-level framework, shipped separately (analogous to SvelteKit / Next.js).
Only if the community asks; the core library stands on its own.

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

**Not planned**: ECS, plugin framework, custom DSL. Trait-based extension
points already cover the real cases.

---

## Unversioned backlog

Rough ideas without a concrete target yet. Promote to a version when active.

- [ ] MSAA edge cases (`resolve_target` in `RenderPassColorAttachments` — beyond current MSAA)
- [ ] Composition mixin-level: merging multiple pass-shaders into one pipeline stage (binding-conflict policy, `use … remap(0, 1)` syntax)
- [ ] Public preprocessor API: `Shader::preprocess(source) -> String` for inspection / shipping pre-resolved shaders
- [ ] Registry: signed uploads to deter namespace squatting
- [ ] Halve conversion traits with `Borrow<B>` across `Into<UniformData>` and `Into<ColorTarget>` paths
