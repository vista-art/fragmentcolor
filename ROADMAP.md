# Roadmap

**Versioning policy (pre-1.0).** Before 1.0, the API is subject to drift and can introduce breaking changes in minor versions as our needs evolve:
- **MAJOR**: Stays at `0.x.x` for the foreseeable future.
- **MINOR**: Breaking changes bump the minor before `1.0.0`. A minor version generally marks a theme milestone (e.g. adding new target platform), a fundamental revamp (switching core dependencies), or redesigning API objects.
- **PATCH**: General bug fixes, non-breaking API additions, dependencies upgrade, build system improvements.

**Sequence.**

1. **0.12.x:** fixes, shader support and tooling that keep today's public signatures.
2. **0.13.0:** one breaking minor with every break and API addition that the earlier plan spread from 0.13.0 to 0.14.x: the primitives an engine drives, instances that live on `Mesh`, the 3D and scene layers with draw merging, plugins, live shader editing, the dependency features, and the record and enum changes the reference scenes need.
3. **0.13.x:** patches that break no public API: fixes, USD loading, the feature tracks, which only add calls (characters and the 2D layer among them), the Mario walk-through and the first integrations.
4. **0.14:** light and surfaces, the next breaking minor: PBR lighting designed once, targets with options, the texture shapes and formats 0.13.0 leaves out, and pipeline completion, with a pbrt-v4 subset rasterized as its acceptance test.
5. **Later:** work that waits on an outside event, such as a split trigger, BoltFFI's rewrite, a stable API for the C target, or a user's request.

0.14 is planned around that one theme, and a break that comes up during 0.13.x waits for it. The Reference scenes goal below ties three milestones to these releases.

Open decisions, further down, lists the choices still open, each with a recommended answer that the sections below follow until it is settled.

## 0.12.x: fixes and groundwork

0.12.6 brings the binding and renderer fixes listed in the CHANGELOG.

### 0.12.7

Items found while reviewing 0.12.6.

- [ ] Lift the `wasm-bindgen` pin (`=0.2.118`): `impl_js_bridge!` hands `RefFromWasmAbi` a raw `u32`, and 0.2.126 changed that ABI to `WasmPtr`. Update the macro, or move it to wasm-bindgen's public `TryFromJsValue`
- [x] `headless_window` is the constructor `HeadlessWindow::new`, so the public API has no free function. Only the window-target doctests and tests use it (0.12.7)
- [ ] Remove `Renderer::create_external_texture` until it works. Every binding returns "not implemented yet", and an API that does nothing doesn't belong in the public surface
- [x] Doc comments that described planned behavior state what the code does: `SceneSource` in `src/scene/loader.rs` and `set_sampler_options` in `src/texture/platform/mobile.rs` (0.12.7)
- [x] The Swift transpiler picks `try` and `await` from the overloads a call's arguments fit, by count and by the shape of each argument, where it merged `throws` across every overload of a name. The generated examples compile with no `try` or `await` that Swift warns about (0.12.7)
- [ ] The public `From` conversions between `wgpu::Extent3d` and `Size` go, as rule 14 asks. `ScreenRegion`'s went in 0.12.7
- [x] The website's Rust tabs leave a doctest's `#` lines out, as rustdoc does (0.12.7)
- [ ] `UniformData` can be named outside the crate, so a helper can forward one value to `Shader::set` on several shaders
- [x] uniffi no longer warns that `uniffi.toml` looks like an old-style config file. The build tools pass no `--config`, and uniffi reads the crate's own `uniffi.toml` from the crate root (0.12.7)
- [x] The Kotlin transpiler keeps a list written one item per line in that layout, comments included, where it flattened a list of shader slugs into `SHADER,))` (0.12.7)
- [x] A pass that draws models and sets no depth target draws with a pooled depth buffer that matches its color attachment, and a depth target the app sets wins
- [x] Swift extensions throw where they stopped the app with `preconditionFailure` on input they can't read, as rule 5 asks: `Vertex.set` and `Instance.set` since 0.12.6, and `setViewport`, the `ScreenRegion` inits and `TextureRegion.from` since 0.12.7
- [x] `Texture::resize` for textures a pass draws into, in every binding. Clones, pass targets, shader uniforms and `TextureTarget::texture()` handles follow it and `TextureTarget::resize`
- [x] Clustered lights with no fixed cap. A compute pass sorts each pass's lights with a range into a grid of cells over its camera view, and a fragment shades only its cell's lights and the lights without one. WebGL2 and native OpenGL take the lights through a uniform block sized to the device. The cap comes from the device, and the storage `lights` and `light_grid` roots are what a custom shader declares (0.12.7)
- [ ] A push that changes only the website reruns every healthcheck. The `paths` filter on `pull_request` compares the whole PR with its base, not the last push. Skip a check when the files it reads match a commit where it already passed, for example with a cache key built from the tree hash of those paths, and keep the skipped job reporting success for branch protection

### First, because 0.13.0 needs them

- [x] Per-language hidden blocks in `docs/api` examples, so each language's platform setup moves from the transpilers into the docs page (0.12.6)
- [ ] Runtime-sized storage arrays (`array<T>`), so one compute shader serves every length through `arrayLength`. `Buffer` needs them, and `convert_type` rejects them today. They ship with the 0.13.0 length rule, a whole number of elements
- [x] Atomics in storage buffers reflect as their scalar, so `layout()` and `set` cover every type from the start (0.12.6)
- [ ] The `#[api]` trial, on a side branch: one attribute on `Shader` that writes the four binding layers, their docs and the Swift and Kotlin overloads. It lives in a `fragmentcolor-macros` crate plus a small rules crate that `build.rs` also reads. On a no-go, 0.13.0 gets hand-written binding code

### Builds and docs

- [x] Build from read-only sources, so docs.rs documents the crate again (0.12.6)
- [ ] Confirm a green docs.rs build once 0.12.6 publishes
- [ ] Declare naga's `wgsl-out` feature, which the GLSL path uses. `glsl-out`, which `process_overrides` needs, is declared since 0.12.7
- [x] Validation fails on a method without `#[lsp_doc]`, and on a method page whose object has no such method (0.12.6)
- [x] Fix the dead `Shader::get_bytes` link on the `Renderer::read_storage` page (0.12.6)
- [x] Fix the `Model` and `Material` pages, which described a `Model::sync_transform` that does not exist and said models that share a mesh collide, where the renderer instances them (0.12.6)
- [x] Fix two pages about loading: the `Renderer::load` page says `render` calls `load` on native targets only, and the JavaScript `Scene.load` doc comment takes the bytes of a `.glb` file (0.12.6)
- [x] Fix three stale references, on the hidden `create_target_ios` page, in the mobile texture enum's doc comment and in the mobile-methods note in `AGENTS.md`, and correct the trait note in `docs/api/PARITY` (0.12.6)
- [x] Replace the `mesh.model` advice on the custom material page with the `model_0` to `model_3` instance inputs the PBR shader reads (0.12.6)
- [x] The `UniformData` doc comments call `Mat2`, `Mat3` and `Mat4` column-major, as their bytes are (0.12.6)
- [x] `src/pass/AGENTS.md` states that FC holds no host object but a target's canvas and that `render` and `submit` never call host code (0.12.6)

### Bug fixes

- [x] JavaScript `renderer.render([...])` records the whole list as one frame, as Rust and Python do, and `renderer.load([...])` loads every item (0.12.6)
- [ ] A changed storage root uploads only the bytes that changed, where today it uploads the whole root
- [ ] Compute pipelines take immediates, as render pipelines do
- [x] A shader whose meshes have different vertex layouts gets one pipeline per layout, and meshes with equal layouts share one (0.12.7)
- [x] Swift and Kotlin `Scene.load` take glTF bytes as well as a path: Swift `Scene.load(_ bytes: Data)` and Kotlin `Scene.load(bytes: ByteArray)`, so the generated examples pass the bytes their comment describes (0.12.6)
- [ ] The transient MSAA texture survives into a following pass that loads it
- [ ] Extend the Apple split from `compute → compute` to `compute (storage write) → render (sampled read)` in one command buffer
- [x] Render pipelines take the first `@vertex` and `@fragment` entry points, as vertex input reflection and compute do (0.12.6)
- [x] `pass.set_viewport` takes effect: the pass draws inside the region, clipped to its target (0.12.6)
- [ ] FC resolves override defaults with naga's `process_overrides` before the module reaches wgpu. Today naga's WGSL writer prints the placeholder 1 for a `@workgroup_size` that names an override, so the browser runs one thread per workgroup where native runs the default
- [x] A mesh recounts its instances on every upload, so a mesh whose instance list shrinks stops drawing stale trailing instances (0.12.6)
- [x] Python raises `FragmentColorError` for renderer and texture errors, as it does for shader errors (0.12.6)
- [x] `create_target` returns `InitializationError::Error` when the surface reports no formats for the adapter, such as a second canvas on WebGL2 (0.12.6). The case gets its own `InitializationError` variant in 0.13.0, once the error enums are `#[non_exhaustive]`
- [ ] A shader and mesh pair whose models are all hidden draws nothing. Today it falls back to the mesh's own instance buffer and draws one stray instance with whatever buffer sits in slot 1
- [ ] `Scene::passes` lists each pass once. Today a member that another member `require`s is listed twice
- [ ] The glTF loader checks `primitive.mode()` and skips line and point primitives with one warning. Today it reads them as triangles
- [x] Turn on the `gltf` crate's `extras` feature, so `Scene::load` reads files that carry application attributes such as `_ID` (0.12.6). The loader ignores the extras until 0.13.0
- [x] PBR normals follow the cofactor of the model matrix's upper 3x3, `cross(m1, m2)`, `cross(m2, m0)` and `cross(m0, m1)`, so `Model::scale` with unequal factors lights the way the surfaces face (0.12.7)

### Binding parity

These landed in 0.12.6. JavaScript errors and the JavaScript `Shader` constructor change behavior, and the rest keep today's signatures. 0.13.0 finishes the ones that need a break, and removes the aliases they keep.

- [x] A `ScreenRegion` built from one size, such as `[w, h]`, covers exactly that size (0.12.6). Two pairs still read as inclusive corners until 0.13.0
- [x] Python exports its window target as `WindowTarget` (0.12.6). `RenderCanvasTarget` stays as an alias until 0.13.0
- [x] Swift and Kotlin see the texture region record as `TextureRegion`, through a uniffi `name` on `TextureRegionMobile` (0.12.6)
- [x] `TextureUsage` flags are plain numbers in JavaScript, as in WebGPU's `GPUTextureUsage` (0.12.6)
- [x] `PassInput` is internal to the crate, and `get_input` leaves every binding. A pass's settings are write-only, through `load_previous`, `set_clear_color`, `set_viewport` and `set_compute_dispatch` (0.12.6)
- [x] `create_texture` sends bytes, paths and URLs that start with the KTX2 identifier to the KTX2 loader, so JavaScript and Python load KTX2 too (0.12.6)
- [x] Swift and Kotlin name the targets `WindowTarget` and `TextureTarget`, through uniffi `name`s on the mobile wrappers. The old names stay as deprecated typealiases until 0.13.0, and the empty `WindowTarget` and `TextureTarget` classes that no method returned are gone, since the core targets stop deriving `uniffi::Object` (0.12.6)
- [x] `scripts/reflect.rs` reads the names uniffi gives Swift and Kotlin, so the `TextureRegion` stand-in left the transpilers (0.12.6)
- [x] JavaScript exports its window target as `WindowTarget`, and `CanvasTarget` stays as an alias until 0.13.0 (0.12.6)
- [x] Every JavaScript method throws an `Error` object, and `new Shader` throws and `Shader.fetch` rejects on bad input, as in the other four languages. A live editor keeps drawing its last good shader by holding its variable (0.12.6)

### Shader support

- [ ] Reflect vertex inputs declared through a struct. `VertexInput.docs` in 0.13.0 needs them, since naga keeps no doc comments on function arguments
- [ ] 16-bit integer shaders (`i16`, `u16` behind `enable wgpu_int16;`) where the adapter supports them, cherry-picked from `v0.13.0-dev` since it adds no call
- [ ] WGSL parse errors quote the offending source line, cherry-picked from `v0.13.0-dev`. Fields for the line, column and input change `ShaderError::ParseError(String)` and the mobile `FragmentColorError::Shader(String)`, so they wait for 0.13.0

### Internal names

No user sees these names, so renaming breaks nothing.

- [x] Qualifiers go first: `TargetInternal` becomes `FrameSource`, the Kotlin transpiler helpers `array_inner` and `floatify_if_numeric_inner` get natural names, and `copy_example_js` becomes `copy_examples` (0.12.6)
- [ ] The transpiler's JavaScript and Python helpers move into `js` and `python` modules in `scripts/convert.rs`, so names such as `camelize_method_calls_js` lose their suffix
- [x] The private Kotlin object `RendererJni` becomes `SurfaceBridge`, together with the Rust `#[jni_fn]` paths and the hidden `create_target_android` page that call it (0.12.6)
- [x] `AGENTS.md` states that qualifiers go first, as in `MobileTextureRegion`, never `TextureRegionMobile` (0.12.6)
- [x] `SceneInner` becomes `SceneState`, like `ModelState` (0.12.6)

## 0.13.0: one breaking minor

Every public call ships in all five languages in this release, and every platform ships the full package.

Every heavy item stays in this release: the 3D and scene layers, plugins, freeing on drop, non-blocking `render`, storage roots owned by their shader and the dependency features. Plugins only add calls, so they may slip to 0.13.1 if they are the last item left. Any other item that slipped would wait for 0.14.

The breaks, detailed below: `add_mesh` on `Pass` takes a shader and an instance range; instances live only on `Mesh`, `add_instance` returns an index, `resize_instances` replaces `add_instances`, `clear_instances` and `set_instance_count`, and `Instance` goes; a mesh lays out its instance slots from the shader in location order, and `set_source` lays them out again; `list_keys` and `list_uniforms` go; `set` takes exact byte lengths and one key grammar; `ScreenRegion` reads two pairs as origin and size; `Compare` replaces `CompareFunction`; `SamplerOptions` and `TextureOptions` gain fields and `TextureFormat` gains cases; `create_external_texture` goes; the pool types turn internal; the aliases and deprecated forms that 0.12.6 keeps for old names go in every language, and `TextureInputMobile` becomes `TextureData` on Swift and Kotlin; sizes, regions and colors become arrays, JavaScript records plain objects, and `TextureId` and `TextureUsage` plain numbers; three free functions become methods; parse errors gain fields, and every public error enum and every record FC returns turns `#[non_exhaustive]`; `Shader::new` rejects ambiguous entry points; core fills five magic uniforms; storage roots belong to their shader; GPU objects are freed on last drop; `render` stops blocking, `Renderer::load` becomes optional warm-up and the `Material` texture setters take a `Texture`; neighboring entries merge, so a hand-ordered list can cost more draws than in 0.12; `Pass::add` keeps its objects in that pass only; `Scene` keeps one pass and `Scene::load` turns async; the glTF loader builds a `SceneNode` per glTF node with a `Model` per primitive over meshes the nodes share, and `scene.nodes()` returns the root nodes; `Model::new` takes references and returns a `Result`, and each instance draws at the model's transform times its `model` slot; `Camera` and `Light` stop writing into shaders; `SceneObject` is sealed and moves into core; `SceneLoadError` and `TextureError` stop wrapping decoder errors; `App` moves to the examples; and the glTF, image and KTX2 decoders move behind default features.

### Before coding starts

- The four 0.12.x items under "First" are done, and the `#[api]` trial has its go or no-go
- Every open question that fixes a 0.13.0 signature or rule is settled: the ones on scene nodes (Open decisions), and the remaining questions on `Pipeline`, shared data and the bindings
- `v0.13.0-dev` is cut again from the 0.12 line after 0.12.6 publishes, since it lacks 378 commits of `v0.12.6-dev`. Its HDR commit carries over, and the 16-bit integer and parse message commits go to the 0.12 line

### Build order

1. **Layers and build surface.** The layer check, the core traits that `Pass::add` and `render` take, the dependency features with `exr`, reflect's feature list and the CI jobs. The proposed `#[api]` emits code under its cfgs
2. **Bindings and names.** The scene types move to `#[api]` with the rest, the glue that lists their concrete types leaves the core modules, and `create_external_texture` goes
3. **Value conversion and the key grammar for `set`.** Needs runtime-sized arrays
4. **`layout()` and `pack()`.** `Buffer`, `Mesh`, magic uniforms and override constants read it
5. **`Pipeline`, override constants and the record changes.** Overrides need 3 and 4, and `camera.sort` reads `Pipeline`. `SamplerOptions`, `TextureOptions` and `TextureFormat` change in the same step, and the error enums and returned records turn `#[non_exhaustive]`
6. **`ScreenRegion` and viewports.** The magic `resolution` reads the viewport
7. **`Buffer`.** Needs 3 and 4. Storage ownership and freeing on drop share its registry code
8. **Instances on `Mesh`.** Needs 3 and 4. `VertexInput.docs` waits for struct inputs
9. **Draw list on `Pass`, with neighbor merging.** Needs 5 and 8
10. **`submit` and `initialize`.** `render` stops blocking, and `Renderer::load` becomes warm-up
11. **Identity and live editing.** Needs 7 and 9. Magic uniforms need 4, 6 and the clock from 10
12. **Plugins.** Need 2 and 11
13. **3D and scene layers.** Needs 1, 5, 8, 9 and 10. The kept group and the loaders sit on the merge from step 9, and the loader fixes ship here
14. **Docs and proof.** The Bevy example is the acceptance test for the release

### Layers

Core holds `Renderer`, the targets, `Shader` with `Pipeline`, `Pass`, `Mesh`, `Texture`, `Buffer`, `layout()` and `pack()`, and it names no 3D or scene type. The 3D layer (`Model`, `Material`, `Camera`, `Light`) calls only the core public API, and the scene layer (`Scene`, `SceneNode`, the loaders) calls only the layers below it. An app adopts the layers it needs and never has to import `Scene`. Every package ships all three. The 2D layer, a 0.13.x track, sits beside the 3D layer under the same rule.

- [ ] A `build.rs` check, next to the no-panic scan, fails the build when a core module names a 3D or scene item, or when a higher layer reaches past the public API of the layers below. It exempts the `SceneObject` seal, which the 3D and scene layers both implement, and binding glue: the code that `build.rs` and `#[api]` generate, or each module's `platform/` files on a no-go. Today core still names higher types in `src/pass`, `src/shader` and `src/renderer`
- [ ] Integration points are small core traits that the higher layers implement. `Pass::add` and `Pass::remove` take `SceneObject`, which moves from `src/scene` into `src/pass` as a sealed trait, and `render(&scene)` goes through `Renderable`
- [ ] The mobile `RenderableHandle` and `SceneObjectHandle` enums, which list concrete types because uniffi cannot take a generic `&impl` argument, move into the binding glue
- [ ] `Model`, `Camera` and `Light` move from `src/scene` into 3D modules beside `src/material`, so the check reads each layer as a set of modules
- [ ] `AGENTS.md` and `src/pass/AGENTS.md` state the layers and the check that enforces them

### Build surface

- [ ] Cargo features gate dependencies only, and no feature removes a type, a variant or a method. The dependency features are on by default: `gltf`, `obj`, the `image` and `ktx2` codecs, `exr`, `network`, and `usd` once it lands. Today `image`, `ktx2` and `gltf` are required. The `shaders-*` features keep embedding the registry's WGSL
- [ ] An `exr` decoder, through the `image` crate's `exr` codec, as a default dependency feature. The `image` codecs are `jpeg`, `png`, `bmp` and `hdr` today. It adds no call, and every environment map and reference image in the first pbrt-v4 scenes is an EXR
- [ ] With a decoder off, `Scene::load` of that format and `create_texture` of that codec return a typed error that names the feature, the way a URL returns `NetworkError::feature_disabled()` without `network`. Each error type has one such case, which later decoders such as `usd` and `basis` reuse, so a patch adds no variant. `SceneLoadError` and `TextureError` carry the decoder's message instead of wrapping `gltf::Error` and `image::ImageError`
- [ ] `network` joins the defaults, which reverses the rule in `Cargo.toml` that keeps a TLS stack out of default builds. ureq builds only on Linux, macOS and Windows, so every `network` gate names those systems, as `agent()` does. Today `From<ureq::Error>` checks only `not(wasm)` and would break the iOS and Android builds. Mobile keeps returning an error, with a message of its own, since `FEATURE_DISABLED` blames a feature that is now on. Every desktop wheel then compiles native-tls, so a dry run of the wheel matrix checks that its six manylinux and four musllinux targets in `publish_py.yml` find OpenSSL, which `Cargo.toml` promises only for maturin's manylinux images
- [ ] `mobile` and `uniffi-cli` gate uniffi, which builds on every target but wasm today, and the `mobile` cfg alias in `build.rs` also requires `feature = "mobile"`. Like `python`, they gate a binding dependency and its glue
- [ ] Every packaging script turns on every dependency feature: `tools/build_web`, which drops the defaults to leave out the embedded registry and so must name each decoder, the maturin list in `pyproject.toml`, `tools/build_ios` and `tools/build_android`
- [ ] The feature list in `scripts/reflect.rs` follows the new features: the dependency features hold in every binding build, `mobile` on iOS and Android, and `winit` and `uniffi/cli` leave. A feature name with no entry fails the build. Today an unknown feature counts as compiled, and so does its `not(...)` arm
- [ ] Validation also fails on a public struct with neither `#[lsp_doc]` nor `#[doc(hidden)]`, as it does for methods since 0.12.6
- [ ] CI runs clippy with no default features and once per dependency feature (`cargo hack clippy --lib --each-feature` over `gltf`, `obj`, `image`, `ktx2`, `exr` and `network`), so a source build with fewer dependencies keeps compiling. Today clippy runs only with `--all-features`
- [ ] `docs/guides/cargo-features.md` lists every feature, the error its absence returns, and the source build for each package with fewer features. It says why no slim package exists: stripping the scene and 3D exports from the 0.12.5 wasm saved 131 KB, 6.8% of its gzip size, and Rust's linker already drops the layers a Rust app never calls
- [ ] A `build.rs` check, next to the no-panic scan, that the shader front end names no `wgpu::` type and no other FragmentColor module
- [ ] The winit `App` and the `winit` feature move into the examples, the dead `dev` alias in `build.rs` goes, and so does the unused PyPI `gltf` dependency in `pyproject.toml`

### Bindings from one attribute

- [ ] On a go, every public type moves to `#[api]`, which carries the `docs/api` text into each binding and generates the Swift and Kotlin overloads. `fragmentcolor-macros` and the rules crate publish with 0.13.0, with a dry-run publish job on pull requests. The `_js`, `_py` and `_mobile` shims and their hidden pages go
- [ ] On a no-go, `create_target_ios` and `create_target_android` become `wrap_layer` and `wrap_window`, and hidden pages move to `hidden/<platform>/`
- [ ] Each flexible parameter names an input enum (`#[api(input)]`), so `Shader::new([...])`, `set(key, value)` and `render([...])` read the same in every language
- [ ] Sizes, regions and colors are arrays in every language, in the generated bindings as well as the extensions. Since 0.12.6 the Swift and Kotlin extensions take `[64, 64]` and `listOf(64u, 64u)` for a size
- [ ] JavaScript records are plain objects when possible, with TypeScript interfaces generated from the Rust structs. Records FC returns are frozen, and readers also accept today's class instances:
  - Plain objects: `Pipeline`, the `layout()` family that `Shader`, `Mesh` and `Buffer` return, `Size`, `ScreenRegion`, `Color`, `SamplerOptions`, `TextureOptions` and `HdrInfo`
  - Plain numbers: `TextureId`, whose class reads its `u64` as a `BigInt` today, and `TextureUsage`, whose flags are already numbers on its class
  - `TextureRegion` stays an array or a plain object in JavaScript and Python, never a class. Swift and Kotlin keep its `from(...).withStride(...)` builder
  - Classes, because they are handles or carry methods: `Camera` and `Light` (`look_at`, `pack`, `sort`), `SceneNode`, `Vertex`, whose chained `set` fills a map the object hides, and the index view that `mesh.instance(7)` returns
- [ ] Object methods replace JavaScript's `set_log_level` and Python's `rendercanvas_context_hook`. Kotlin's top-level `ShaderFetch` goes with the other deprecated forms under Names
- [ ] Python releases the GIL in every blocking call. Today nothing calls `py.detach`
- [ ] Parse errors carry the line, column and input as fields in every binding
- [ ] HDR and wide-gamut surface output from `v0.13.0-dev`: a `ColorSpace` enum plus `color_spaces()`, `set_color_space()` and `hdr_info()` on window targets, with `Auto` as the default

### Names

0.12.6 gives Swift and Kotlin the core names for the targets and `TextureRegion`, and gives JavaScript and Python `WindowTarget`, each with an alias for the old name. 0.13.0 removes every alias.

- [ ] The deprecated Swift and Kotlin typealiases go: `MobileWindowTarget`, `MobileTextureTarget` and `TextureRegionMobile`
- [ ] The other forms 0.12.6 deprecates go too: Swift's `Shader.new([...])` and `Shader.setRegistry(baseUrl:)`, its one-tuple `createTexture` and `createStorageTexture`, and Kotlin's top-level `ShaderFetch`
- [ ] JavaScript's `CanvasTarget` alias and Python's `RenderCanvasTarget` alias go
- [ ] Swift and Kotlin see `TextureData` for `TextureInputMobile`, through a uniffi `name`. `stand_ins` in `scripts/reflect.rs` then drops the pair, so the stand-in rewrite for `TextureData` leaves the transpilers
- [ ] The Swift and Kotlin `TextureRegion` fields are `x`, `y`, `z`, `width`, `height` and `depth`, the keys JavaScript and Python read
- [ ] Rust idents put the qualifier first: `MobileTextureRegion` and `MobileTextureData`
- [ ] The mobile `SceneObjectHandle` becomes Rust `MobileSceneObject`, exported to Swift and Kotlin as `SceneObject`. `PassTarget` leaves with `PassRef`

### `Pipeline` on `Shader`

- [ ] One `Pipeline` record with `blend`, `cull`, `topology`, `depth_test`, `depth_write`, `depth_bias`, `color_write`, `front_face` and `alpha_to_coverage`, whose default matches today's pipeline. The last four serve shadow maps, depth prepasses, mirrored meshes and cutouts such as leaves
- [ ] `shader.set_pipeline(pipeline)` replaces the record, and `shader.pipeline()` returns a copy. Compute shaders ignore it, and setting it never fails
- [ ] Presets as static constructors: `Pipeline::opaque()`, `Pipeline::transparent()` and `Pipeline::additive()`. In JavaScript they return plain objects, so `{ ...Pipeline.transparent(), cull: Cull.Back }` works
- [ ] Enums `Blend { Off, Alpha, Premultiplied, Additive }`, `Cull { Off, Front, Back }`, `FrontFace { CounterClockwise, Clockwise }`, `Topology` and `Compare`, which renames `CompareFunction` and gains `Hash`. `SamplerOptions.compare` keeps its field name
- [ ] The pipeline cache key carries the record, and strip topologies set a strip index format
- [ ] Before any wgpu call, FC checks override values, blendable formats and the device's features, each with a typed error, and caches a failed build under its key
- [ ] Pipeline creation runs inside `validate`, so wgpu failures return typed errors on native and WebGL2
- [ ] Every public error enum turns `#[non_exhaustive]`, and so does every record FC returns and users never build: the `layout()` records and `HdrInfo`. A patch can then add a variant or a field without breaking callers, and a field it adds carries a uniffi default. No FragmentColor type has the attribute today. The surface that lists no formats then gets its own `InitializationError` variant
- [ ] `Material`'s `alpha_mode` and `double_sided` become sugar over `set_pipeline`

### Textures and samplers

- [ ] `SamplerOptions` gains `anisotropy` and a mirror wrap mode. It holds `repeat_x`, `repeat_y`, `smooth` and `compare` today, and the glTF loader reads `MIRRORED_REPEAT` as plain repeat. Floors blur at grazing angles without anisotropy
- [ ] A texture uniform binds a view of the dimension its slot declares, so `texture_cube`, `texture_cube_array` and `texture_2d_array` slots take cube and layered textures. `Renderer::render` refuses those slots with `BindingError` today, since every view covers the whole texture
- [ ] `TextureOptions` gains `dimension` (2D, cube, array or 3D) and a layer count, picked from the container when unset. `TextureFormat` gains the BC family, ASTC (4×4 to 12×12, Unorm, UnormSrgb and HDR), ETC2 and EAC, which the KTX2 path reads as `wgpu::TextureFormat` today. Cube, array and 3D uploads from KTX2 follow as 0.13.x patches
- [ ] `create_external_texture` goes from every binding, with its pages. Each returns an error today, and reshaping it later would break callers. Video without a copy returns as a new call once wgpu supports it everywhere

### WGSL `override` constants

- [ ] `shader.set(name, value)` and `shader.get(name)` work on `override` declarations, and the next `render` or `submit` builds a pipeline with the value
- [ ] FC resolves the values with naga's `process_overrides` before wgpu sees the module, on every target, and passes wgpu an empty constants list
- [ ] `layout().overrides` lists `ShaderOverride { name, wgsl, required, docs }`. A required override without a value returns `RendererError::MissingOverride`
- [ ] Both pipeline cache keys carry the override values. FC logs one warning when one shader reaches 16 pipelines
- [ ] `pack` and `Buffer` binds return an error on an override key

### One value conversion and one key grammar for `set`

- [ ] Numbers spread over struct and array keys, in declaration order
- [ ] Bytes on any key, raw bytes into uniform struct roots included. They must fit the key exactly: the full size of a fixed-size key, or a whole number of elements for a runtime-sized array. Today long input is cut and short input padded with zeros
- [ ] One key grammar on `Mesh`, `Buffer` and `Shader`. `[7].tint` is one instance, `[200:250].tint` a half-open span and `[200:].tint` runs to the end. `tint` with no index writes every instance and the start value new ones take. `Shader` keys start with the root: `particles[7].pos`, `particles.pos`. Today a key reaches one element at a time (`data.arr[3].field`). `tint`, `pos` and `particles` are example names a shader would declare
- [ ] A value fills its key exactly: one value, or the span length times the slot size in bytes, and a bare key's one value copies everywhere. `set` never changes a count, an index past the end is a typed error, and instance keys end in a slot name. A bare `get` reads the start value: the identity for a matrix, zero otherwise
- [x] A value converts by its slot's WGSL type, so a plain `7` fills a `u32` in every language. `Shader::set` converts uniforms, and `Vertex::set` and `Instance::set` values convert when the mesh attaches (0.12.6)
- [ ] `set`, `pack` and the `Mesh` and `Buffer` methods share this conversion

### Reflection with `layout()` and `pack()`

- [ ] `shader.layout()` returns a `ShaderLayout { bindings, inputs, overrides, workgroup_size }`. It is built once from the parsed module and cannot fail
- [ ] `ShaderBinding { name, kind, group, index, wgsl, size, writable, fields, docs }`, `ShaderField { key, wgsl, offset, size, count, stride, docs }`, `VertexInput { name, location, wgsl, size, docs }` and `BindingKind { Uniform, Storage, Texture, StorageTexture, Sampler, Immediate }`. An input's `docs` comes from its struct member
- [ ] `layout()` is the one discovery name: `mesh.layout()` lists vertex fields and instance slots with offsets and strides, and `buffer.layout()` lists the fields of the root it is bound to
- [ ] `workgroup_size` reports a dimension that names an override with its current value, and 0 while a required override has no value
- [ ] `shader.pack(key, value)` returns the bytes `set(key, value)` would write, without reading or changing the shader
- [ ] `pack` takes binding keys only. Instance slots are written through `mesh.set` and `write_instances`, so input names never share `pack`'s keys
- [ ] `list_keys` and `list_uniforms` are removed. Root names come from `bindings[].name` and keys from `fields[].key`
- [ ] The renderer's bind loops read the cached layout
- [ ] One public word for push constants: "immediates", in `BindingKind` and in the docs

### Running without a target

- [ ] `renderer.submit(&renderable)` runs compute passes and passes that have their own targets, with no frame to acquire. It is sync in every language, like `render`
- [ ] `renderer.initialize()` creates the device for jobs with no target. It is async in Rust, JavaScript, Swift and Kotlin, blocks in Python, and apps with a target never call it. On WebGL2 before any canvas target, it fails with a message that names `createTarget(canvas)`
- [ ] Every `Renderer` call that waits creates a missing device, and `render` and `submit` never do. `read_texture` returns `NoContext` today. The `NoContext` message names `initialize` and `create_target`
- [ ] Before encoding anything, `submit` checks the pass graph (`NoTarget` when a pass needs a frame), then the device (`NoContext`), then compute support. `render` makes the same compute check
- [ ] `render` acquires a frame only when some pass draws into it. Since 0.12.6, a render of compute passes alone already skips it
- [ ] Textures from the create calls finish uploading inside those calls, which wait, so `render` never blocks on them. Today native `render` blocks on `load` first
- [ ] `Renderer::load` keeps its signature as optional warm-up for any renderable. It creates a missing device and uploads every texture and mesh the renderable draws, with their vertices and instances. Pipelines still build in the first render, since their key holds the target's formats and sample count. `render` uploads whatever `load` skipped with queued writes
- [ ] The `Material` texture setters stop queuing uploads for `render`. Proposed: they take a `Texture` from `create_texture`

### `Buffer`

A `Buffer` holds uniform and storage roots. Instance data lives on `Mesh`, so a `Buffer` never feeds a vertex stream.

- [ ] `Buffer::new()`, `buffer.set(key, value)` and `buffer.get(key)` with keys relative to the root, `buffer.write(bytes)`, which returns an error when the length is wrong, `buffer.size()` and `buffer.layout()`, which is empty before the first bind
- [ ] `shader.set("camera", &buffer)` binds one buffer to a uniform or storage root in any number of shaders. The buffer takes its layout from the first root it binds to, and later binds must match it
- [ ] A plain `shader.set` on a bound root writes through to the buffer, and `shader.get` reads from it
- [ ] A uniform root bound to a buffer uploads once per render, however many shaders bind it
- [ ] A storage `Buffer` bound to an `array<T>` root takes the shared key grammar: `[7].glow`, `[200:250].glow`, `[200:].glow`, and a bare `glow` for every element (`glow` is an example field)
- [ ] Storage roots belong to their shader. A `Buffer` is the one way to share them, and `read_storage` reads it back through a shader that binds it. Today the renderer shares them by root name
- [ ] One GPU copy per device for `Buffer` and `Mesh`
- [ ] GPU objects are freed when their last handle drops, and `unregister_texture` becomes an early release
- [ ] A pass error when one buffer is read-write storage and also has another role in the same pass
- [x] `BufferPool` and `PoolStats` are gone, and `TexturePool` and `RenderContext` are internal (0.12.6)

### `Mesh`: vertices and instances

Instances live on `Mesh`, and only there. `tint` below is an example slot a shader would declare. The one slot name FC reads is `model`, the transform.

- [ ] A new mesh holds one instance. `add_instance()` appends one from the start values and returns its index, `remove_instance(index)` keeps order, and `resize_instances(count)` grows from the start values or cuts from the end, where `0` draws nothing. `instance_count()` reads the length. Today `add_instance` takes an `Instance`, and `set_instance_count` overrides the count
- [ ] `write_instances(bytes)` replaces every instance in the mesh's layout and takes the count from the length
- [ ] `write_vertices(bytes)` replaces every vertex in the mesh's layout and takes the count from the length. It keeps `set_indices` indices that fit, draws in order without them, recomputes bounds and reuses the GPU buffer while the bytes fit. Each `Vertex` holds two `HashMap`s keyed by `String` today, and the one mesh in the `ganesha` scene has 2,360,548 vertices
- [ ] `mesh.set(key, value)` and `mesh.get(key)` address instance slots through the key grammar, and `mesh.layout()` describes both stores
- [ ] Index views, as documented sugar: `mesh.instance(7)` holds the mesh and an index with no tracking, and its `set("tint", v)` is exactly `mesh.set("[7].tint", v)`. `mesh.instances()` iterates the views
- [ ] `mesh.share()` returns a mesh over the same vertices and indices with a copy of the instances, its own from then on
- [ ] The shader is the schema. A slot is a `@location` input the vertices don't supply, typed by its WGSL declaration. Four `vec4<f32>` inputs `name_0` to `name_3` at consecutive locations form one `mat4x4<f32>` slot `name`, column-major, set as `[[f32; 4]; 4]`. The transform keeps the inputs `model_0` to `model_3`
- [ ] `position` and the `Vertex` constants never become slots, so a mesh that lacks `tangent` still fails at add time
- [ ] `joints0` and `weights0` join those constants as documented vertex keys, so meshes built by hand and the 0.13.x skin loader agree
- [ ] The first check (`pass.add_mesh`, `shader.add_mesh`, `shader.validate_mesh` or `Model::new`) lays the mesh out in location order. Today the first vertex and the first instance set the layout, sorted by name. Later shaders may read a subset of the slots, and any other mismatch is a typed error at add time that names both shaders
- [ ] Each store reuses one GPU buffer while it fits and uploads only the bytes that changed. Today instances repack from `Instance` maps
- [ ] `add_instances`, `clear_instances`, `set_instance_count`, `Instance`, `Vertex::create_instance` and `From<Vertex> for Instance` go, and JavaScript objects and Python dicts stop serving as instances
- [ ] `Vertex::set` stays a `#[must_use]` builder by value in Rust. The bindings write in place and return the same object, so no dropped result loses a write
- [ ] Drawing by count alone is `quad.resize_instances(4096)` under a shader with no slots

### Draw list on `Pass`

On `Pass`, `add_mesh` and `add_shader` add a draw, and `add` keeps a `SceneObject` whose draws the pass rebuilds at each render.

- [ ] `pass.add_mesh(&mesh, &shader, range)` draws a range of the mesh's instances with that shader, in this pass only. Today `add_mesh(&mesh)` attaches the mesh to the last shader in the pass that accepts it and drops the other errors, which changes every pass that uses that shader
- [ ] The range is a plain instance range: `..` for every instance or `a..b` in Rust (`impl RangeBounds<u32>`). Elsewhere it is optional, and leaving it out draws every instance: `[a, b]` in JavaScript, `range(a, b)` in Python, `a..<b` in Swift and `a until b` with `UInt` in Kotlin. `@builtin(instance_index)` is the instance's index in the mesh
- [ ] `scripts/reflect.rs` marks the range parameter, so the transpilers drop a trailing `..` and turn `a..b` into each language's range
- [ ] A range clamps at render time against the mesh's instance count and logs once per entry, so one bad range never drops the whole render
- [ ] `add_mesh` checks at add time that the shader's vertex inputs fit the mesh and lays out or checks its instance slots, as `src/shader/AGENTS.md` requires. `shader.validate_mesh` makes the same check without adding an entry
- [ ] The shader's mesh list stays. `pass.add_shader` draws the shader's own meshes, every instance of each, where the shader sits
- [ ] `remove_mesh`, `remove_shader`, `clear_meshes` and `clear_shaders` on `Pass`. There is no `add_range`
- [ ] Entries run in the order they were added, and FC never reorders them. A group of models that a pass keeps through `pass.add` is the one place FC sorts
- [ ] Neighboring entries that share a mesh and a shader merge into one instanced draw, and touching ranges merge with no copy. Entries that draw nothing are skipped first
- [ ] Kept models that share vertices, a shader and a layout pack into one instance buffer per pass, kept across frames and rewritten only where a model, its transform or its mesh's instances changed. Packing and sorting apply only when the vertex stage never reads `instance_index`. Today every frame creates a buffer per opaque group and per blended model
- [ ] Blended neighbors merge on backends that pass a pixel test of blend order within one draw, and draw one at a time elsewhere
- [ ] A mesh without indices draws without them. Today every draw is indexed
- [ ] Pipelines keyed per entry layout, a validation cache per shader and mesh layout, and uniform pools sized from the distinct shaders in a pass
- [ ] Tests on every backend for a non-zero first instance, blend order within one draw, and merged runs that drop or shift no instance

### `ScreenRegion` and viewports

- [ ] Two pairs mean origin and size everywhere, and a single pair is the exact size, as it is since 0.12.6. Today two pairs read as inclusive corners. The inclusive +1 stays only in `from_pixel` and `encompass`

### Identity and live editing

- [ ] `Eq` and `Hash` on `Shader`, `Mesh`, `Model` and `SceneNode`, by handle, with each language's own equality where it has one
- [ ] `Shader::new` accepts an existing `Shader` and reuses its parsed module, so one WGSL source runs cheaply as two pipelines with different `Pipeline` records or override values
- [ ] `Shader::new` returns an error when a module has two entry points of one stage, or both compute and render entry points
- [ ] `Renderer: Clone`, and compile-time checks that the public handles are `Send + Sync`
- [ ] `shader.source()` returns the composed WGSL, and `shader.set_source(source)` swaps the program in place. On error it changes nothing, and the old program keeps drawing. On success the shader keeps its place in every pass, its `Pipeline`, meshes, `Buffer` binds and every value whose key and type still match. It lays out again the instance slots of each mesh it laid out, keeping every slot whose name and type survive. It fails when the stage kind changes, an attached mesh no longer fits, another shader still reads a slot the edit drops, or a bound root changes layout
- [ ] When one shader's pipeline fails to build at render time, `render` and `submit` skip that shader's entries, run the rest, and return the error. `render` still presents the frame

### Magic uniforms

Today the app fills these names and core fills nothing.

- [ ] Core fills `resolution`, `aspect`, `time`, `delta_time` and `frame` only on a top-level `var<uniform>` with that exact name and type, in every language
- [ ] Any `shader.set` on one of those keys hands it to the app for the life of that `Shader`. `shader.get` reads the last value
- [ ] One clock and one frame counter per `Renderer`, advanced by every `render` and `submit`. `resolution` and `aspect` come from the pass's viewport, else its target, so a compute pass with no target leaves them to the app
- [ ] `mouse`, `scroll` and keys stay with the app and the integrations. The migration section lists the change, because a declared but unset `time` starts to animate

### Plugins

- [ ] Static `Shader.add_plugin(prefix, plugin)` and `Shader.remove_plugin(prefix)`, with `name:` and `@scope/` prefixes, and `http`, `https` and `fc` reserved. A part such as `"llm:a glowing ring"` in the list `Shader.fetch` takes goes to the plugin registered for `llm:`, which returns WGSL
- [ ] Plugins run only inside `Shader.fetch`. Their output is cached in memory once the whole shader compiles, and concurrent requests for one part share a single call
- [ ] `src/pass/AGENTS.md` lists registered plugins beside a target's canvas as the host objects FC holds. `render` and `submit` still never call host code
- [ ] `ShaderError::PluginError { input, cause }` names the failed part and the cause. It also covers a prefix with no plugin, and a plugin part that reaches `Shader::new` uncached
- [ ] Callbacks in every binding: async functions in Rust and JavaScript, plain callables in Python, closures in Swift and Kotlin. A Python plugin's exception becomes `PluginError`
- [ ] The built-in URL, path and slug handling become entries in the same table
- [ ] Doc pages use a deterministic `demo:` plugin, so every healthcheck runs a callback
- [ ] `ShaderError` turns `#[non_exhaustive]`, so plugins may slip to 0.13.1 if they are the last item left

### The 3D layer

`Model`, `Material`, `AlphaMode`, `Camera` and `Light` stay in `fragmentcolor` under their names, each with a `docs/api` page and examples in every language. The layer calls only the core public API.

- [ ] A `Model` is a mesh, a material, a transform and a visibility flag, with no instance calls. It reaches its instances only through `model.mesh()`
- [ ] `Model::new(&mesh, &material)` takes both by reference and returns `Result<Model, ModelError>`. It lays out the mesh's slots through `validate_mesh`, and fails with `ModelError::MissingTransform` when the material's shader has no `model` slot. `ModelError` is `#[non_exhaustive]` and lifts into `SceneLoadError` and the binding errors with `#[from]`
- [ ] Each instance draws at the model's transform times its `model` slot. At the identity the model draws its mesh as is. Otherwise its `SceneObject` hook, at each render, reads `mesh.get("[0:].model")`, composes on the CPU, writes a private `share()` with `set("[0:].model", bytes)` and adds that with `pass.add_mesh`: a product and 64 bytes per instance per render
- [ ] Models on one mesh share its instances and keep their own transforms, in one pass or several. `mesh.share()` gives one of them instances of its own over the same vertices
- [ ] `set_transform`, `translate`, `rotate`, `scale` and `transform()` place the model. Tinted models share one `Material` and keep the tint in a slot of their mesh, so `Model` gains no uniform
- [ ] `model.add_to(&pass)` adds the model's entry now, over every instance of its mesh
- [ ] `set_visible(false)` leaves the model out of the draw and keeps every value, so its neighbors merge around it, and `set_visible(true)` puts it back
- [ ] `model.name()` returns the glTF mesh name, or an empty string
- [ ] `model.share()` returns a model over the same mesh and material with its own transform, flag and name. `SceneNode::share` copies models through it
- [ ] A model's transform, a camera's pose and a light's position and direction are relative to their holder: the world when a pass keeps them, or the `SceneNode` they sit in. The `Model`, `Camera` and `Light` pages say so first
- [ ] `Model` stops writing an instance layout onto the shared `Mesh` and stops queuing `model_entries`
- [ ] `camera.sort(&models)` returns the visible models: opaque ones grouped by shader and then by vertices, then blended ones far to near by the depth of their bounds' center. It reads blending from `shader.pipeline()`
- [ ] `Camera` and `Light` stay handles and stop writing into shaders. A pass that keeps them writes them into its `Buffer`s at each render, and `pack()` returns their bytes in the PBR layout for hosts with their own `Buffer`
- [ ] `Material`'s `alpha_mode` and `double_sided` become sugar over `set_pipeline`, so core reads only `Pipeline` and never names `AlphaMode`
- [ ] `Model`, `Camera` and `Light` implement the core `SceneObject` trait with public calls: a model reports its mesh, shader, transform and bounds, a camera its `pack()` bytes, and a light its entry for `lights`
- [ ] The layer check holds the 3D layer to the core public API and the `SceneObject` seal. Today `src/scene` and `src/material` reach crate internals on 49 lines
- [ ] Rewrite the `Model` pages, add pages for `add_to`, `sort`, `pack` and `name`, and port `model_pbr_triangle`, `transparent_quads`, `gallery_camera` and `gallery_lighting`

### Kept objects on `Pass`

`pass.add` and `pass.remove` are core calls that take the sealed core trait `SceneObject`. The 3D and scene layers implement it, so core holds kept objects without naming their types. The seal is the one crate-private item those layers name.

- [ ] `pass.add(&object)` keeps a `Model`, `Camera`, `Light` or `SceneNode`, a node with everything under it, in that pass until the new `pass.remove(&object)`. A second `add` does nothing, and `clear_meshes` leaves kept objects alone. Today `add` attaches meshes to shaders for every pass, and a second `add` draws twice
- [ ] At each render, a pass that keeps objects writes its last kept camera into a `Buffer` it owns and its kept lights into the clustered light buffers of 0.12.7, which become its own. A light past the device's cap fails its `add` with `PassError::LightCapReached`, as in 0.12.7. While it draws, it binds them to each shader with a PBR `camera` or `lights` root that the app has not bound or set
- [ ] Cameras and lights reached through a kept `SceneNode` count as kept. The pass uses the last camera it reaches, in add order with a node's cameras at the node's place. Lights past the device's cap that reach a kept node after its `add` fail the render with `RendererError::PassError`, as lights added before the device opened do in 0.12.7
- [ ] The pass sorts its kept models with the `camera.sort` rule and draws them as one group where the first `add` sat. Blended instances sort far to near by each instance's composed transform applied to the bounds center, where today each model sorts as one. It rebuilds the group only when a model, a node above it, the camera, a mesh's instances or a `Pipeline` changed
- [ ] In a kept group, instances whose composed matrix has a negative determinant draw in a run of their own with the opposite `front_face`, through a second pipeline for that shader, as three.js and Bevy do. A node scaled by -1 keeps its front faces
- [ ] The bindings take `Model`, `Camera`, `Light` and `SceneNode`, and only binding glue lists them, such as the mobile `Pass.add`
- [ ] The pass fields `model_entries`, `scene_objects` and `camera_snapshot` and the renderer's model path give way to one kept group that the renderer fills through a step behind the seal
- [ ] Rewrite the `Pass::add` page and add one for `remove`
- [ ] The older uniform `LightArray` light root, which 0.12.7 still fills up to its array length, goes. A custom shader declares the storage `lights` root, and `light_roots.rs` drops the older shape

### The scene layer

`Scene` and the loaders stay and `SceneNode` joins them, bound in five languages. They call only the core and 3D layers. A `Model` stays one mesh and one material, and `SceneNode` groups models one level up, as three.js, Bevy, Godot, Unity and Unreal all keep per-part transforms above the mesh. Hosts with their own hierarchy, such as an ECS, never need it.

- [ ] `Scene` keeps one pass, reached through `scene.pass()`, plus `add`, `remove`, `models`, `cameras`, `lights`, `nodes`, `set_ambient` (was `ambient`) and `skip_defaults` (was `no_defaults`). `add_to`, `PassRef`, the six pass list methods and the other four default methods go
- [ ] The typed lists read what the pass reaches, through nodes too, and `scene.nodes()` returns the root nodes the pass keeps. The stock camera and light follow the target's aspect and stay out of them
- [x] The scene's pass draws with a pooled depth target that matches the render target's size and sample count, and a depth target the app sets wins (0.12.7)
- [ ] `SceneNode` is a handle that holds a local transform, a visibility flag, a name, glTF `extras`, models, cameras, lights and child nodes: `SceneNode::new()`, `add(&object)` and `remove(&object)` for a `Model`, `Camera`, `Light` or `SceneNode`, `models()`, `cameras()`, `lights()`, `nodes()`, `parent()`, `transform()`, `set_transform`, `translate`, `rotate`, `scale`, `world()`, `visible()`, `set_visible`, `name()`, `extras()` and `share()`. `name()` and `extras()` return `""` and `"{}"` on a node built by hand
- [ ] Each instance under a node draws at `node.world() * model.transform() * slot`, where `world()` multiplies the transforms from the root down, parent on the left, as glTF composes nodes. A camera under a node sees through its own view matrix times the inverse of `node.world()`, a point or spot light sits at `node.world()` times its position, and a directional or spot light's direction turns with the node
- [ ] A node has one parent. `add` returns a `SceneError` that names the node when it already has a parent or when the add would close a cycle. `SceneError` is `#[non_exhaustive]` and lifts into the binding errors with `#[from]`
- [ ] `set_visible(false)` on a node leaves out every model and light under it, keeps each one's own flag, and keeps its cameras in use
- [ ] `node.share()` returns a copy of the tree with no parent: the same names, extras, transforms and flags, a new `Model` for each model through `model.share()`, and the same `Camera` and `Light` handles. The copies' models share meshes, so 100 copies of a car with a body and four wheel models draw in 2 draws
- [ ] The node's `SceneObject` hook asks the hook of each model, camera and light under it for its entry and applies the node's world matrix. The pass recomposes only a subtree whose node or contents changed, so a still scene does no work after its first frame
- [ ] Rewrite the `Scene` pages, add pages for `pass`, `nodes`, `set_ambient` and `skip_defaults`, for `SceneNode` and its calls and for `SceneError`, and port `gltf_scene` and `model_viewer`

### Loading

- [ ] `Scene::load(source)` is async in Rust, JavaScript, Swift and Kotlin and blocks in Python. It needs no device. It takes a path on native, a URL on the web and on native with `network`, and bytes everywhere, through one call. Today the JavaScript string always fails, and Swift and Kotlin read bytes through a second constructor, `Scene.loadBytes`
- [ ] The format comes from the extension, or from sniffing the bytes. A failed sniff names the formats tried and the features that are off, and a format whose feature is off returns an error that names it
- [ ] One `Mesh` per glTF primitive, built once with one instance and shared by every node that draws it. One `SceneNode` per glTF node, with its local transform, name and `extras`, and in it one `Model` per primitive at the identity, so glTF reuse packs. Primitives over the same accessors share vertices through `share()`. Each glTF camera and light goes into its node at the glTF default pose, the node's origin looking down -Z with +Y up. Each OBJ object becomes a root node with a model per material. Today the loader builds a `Mesh` and a `Model` per primitive per node visit
- [ ] A skinned primitive's model sits at the scene root, since glTF 2.0.1 ignores its node's transform. Mirrored nodes keep their front faces through the pass's mirrored run (Kept objects on `Pass`). Today every primitive takes its node's world transform, and none reverses
- [ ] Each image becomes one texture, uploaded once. Today each material slot clones the pixels into an upload of its own
- [ ] OBJ and MTL through `tobj` 4.0.5, with MTL's Phong fields mapped onto PBR factors
- [ ] A host reads the glTF tree as handles: `scene.nodes()` returns the roots and `node.nodes()` the children, so it walks down and never compares handles, which would fail in JavaScript, where each call returns a new wrapper. An ECS host keeps each model on its own with `pass.add(&model)` and writes world matrices into it, and kept models that share vertices still pack
- [ ] A glTF attribute `_FOO` becomes the vertex key `foo`. A name that equals a built-in key fails the load and names both
- [ ] The Rust example on `docs/api/core/renderer/load.md` warms up a core renderable, since `load` serves any renderable. Today it imports `Material`, `Model` and `Scene`

### Docs and proof

- [ ] Pages for every new call, and override sections on the `set` and `layout` pages. The first line of the `add_mesh` page says a JavaScript pair there is a start and an end, unlike in `ScreenRegion`
- [ ] Pages for the `Mesh` instance calls, `write_vertices`, `write_instances`, `share`, `layout` and the index views, in place of the `Instance` pages
- [ ] Reword the opening of `docs/api/core/pass/pass.md` so a reader can tell the `Shader`, which is the whole pipeline, from its `Pipeline` record
- [ ] One CHANGELOG migration section that maps every break onto its replacement, with a before and after line for each Swift and Kotlin rename. The Swift install snippets in `Package.swift` and `CONTRIBUTING.md` use `.upToNextMinor(from: "0.13.0")`. It says that a hand-ordered list of models that interleaves materials now costs draws that 0.12 saved, and that `camera.sort` restores them
- [ ] Guides: drive FragmentColor from an ECS on the core and 3D layers, never importing `Scene`. Live coding, with the revert pattern. Draw counts, as a ladder from a `Scene` to sorted `add_to` calls to a crowd as the instances of one mesh, with the table of cases that merging cannot reach. Cargo features. Scene nodes, with a car placed 100 times through `share()`, a glTF tree walked into ECS entities, and when a part is a model of its own or an instance of a shared mesh. Magic uniforms, with the rule that the app's value always wins. Custom components, with four cases: a host `Transform` through `Model` with a tint slot on its mesh, a crowd written with one `write_instances` call, an outline drawn as a second entry over the same instances, and a `u32` ID into a storage table
- [ ] A compute tutorial on `initialize`, `submit`, `layout`, `pack`, `read_storage` and `override` constants, and a `read_storage` example on the same calls
- [ ] `examples/interop/bevy`, driven from `bevy_ecs` on the core and 3D layers, and a plain-loop instancing example, both built in CI

## 0.13.x: patches, tracks and integrations

Patches add calls and fix bugs, and none breaks a public signature. A track item that would break one waits for 0.14.

### Hygiene

- [ ] A bind group cache keyed on each `Buffer`'s version and on dirty flags
- [ ] One GPU error scope per frame instead of one per operation
- [ ] A dependency-graph version per renderer instead of one per process
- [ ] An eviction rule for the render and compute pipeline caches, which grow by one pipeline per live-coding edit
- [ ] Python reads bytes and NumPy arrays without copying them

### Formats

- [ ] USD (usda, usdc, usdz) through `openusd`, as a `usd` dependency feature that is on by default like `gltf`, once a license review passes and a trial builds the crate for wasm, iOS and Android. It declares MIT, so cargo-deny would pass it, but its README puts one generated schema file under the Tomorrow Open Source Technology License 1.0, which `deny.toml` does not list. It needs Rust 1.96. USD lights other than directional, point and spot are skipped with one warning
- [ ] Opening `SceneObject` to outside types, once a design keeps the rule that `render` and `submit` never call host code

### Characters

Skinning splits across the layers, and every call is an addition. The host keeps the clock, looping, blend weights, crossfades, state machines, IK, retargeting and root motion, and FC never runs a clock or calls host code. In this order:

- [ ] Data textures from a `Buffer` (core). A `Buffer` binds to a `texture_2d<f32>` or `texture_2d<u32>` root, and core uploads its bytes. Joint palettes, morph deltas and lookup tables then share one WGSL path on every backend, WebGL2 included, which has no storage buffers. A texture 2048 texels wide, WebGL2's limit in wgpu, holds 512 matrices per line
- [ ] The skin loader (scene): `JOINTS_0`, `WEIGHTS_0`, inverse binds, skins, clips and morph weights, read into the `joints0` and `weights0` keys. The locked `gltf` 1.4.1 already reads all of them, and the loader skips them today
- [ ] Playback with no clock or play state (scene): `scene.animations()`, `scene.animate(&clip, seconds)`, which writes the transforms of the `SceneNode`s a clip moves, and `scene.sample(&clip, seconds)`, which returns one TRS record per node for hosts with their own ECS. Keys clamp as the glTF spec says, so the host loops. glTF node animation comes with it and leaves Later
- [ ] Skinned PBR (3D): its own `Material` constructor that the loader picks, a `mesh/skin` registry snippet, a palette range per skinned `Model`, `model.set_joints(&matrices)`, and skinned bounds for `camera.sort`. Every instance of a model's mesh shares its pose unless a per-instance offset slot points elsewhere
- [ ] Morph targets (3D, scene): deltas in a data texture read by `vertex_index`, applied before skinning
- [ ] A glTF sample test matrix at fixed times: BoxAnimated, SimpleSkin, RiggedFigure, CesiumMan, Fox, SimpleMorph, MorphStressTest and InterpolationTest, with attribution where a license asks for it. BrainStem, under the Poser EULA, stays out

### The 2D layer

A sibling of the 3D layer in every package, built on core only and checked by the same layer rule. A sprite is one instance of a shared quad `Mesh`, with `rect`, `source`, `tint` and `angle` slots that the `sprite/quad` shader declares. Neighbor merging draws sprites that share a sheet in one call, in the host's order. The layer adds no `Sprite` handle, 2D camera or batch type. The host keeps entity order, y-sorting, input and UI layout.

- [ ] `sprite/quad` with a `view` uniform (`offset`, `zoom`, `angle`), as a registry pass shader. It needs the composition track's pass shader entry kind. The categories are `sprite/` and `text/`, because `2d/` fails the slug grammar
- [ ] `Atlas` (2D) as data: TexturePacker and Aseprite JSON, grids, `get` and `sample(tag, seconds)`. It parses through a `json` dependency feature that is on by default and removes no type
- [ ] `Font` (2D) as data, over baked BMFont and msdf-atlas-gen files: it lays a string out as glyph instances of the quad mesh, measures it and reports its distance range. The `text/bitmap`, `text/sdf` and `text/msdf` shaders draw it. Baked fonts come first, and shaping stays with the host or an integration
- [ ] `sprite/slice`, `sprite/shape` and `sprite/grid`, then a tilemap example that reads tile indexes from a texture
- [ ] A `sprite/quad` variant with a `layer` slot over an array texture, on 0.13.0's layer count, so sprites from several sheets merge into one draw
- [ ] Golden images compared against PixiJS v8 drawing the same sheet and label

### Reference scene work

The Reference scenes goal below holds the detail.

- [ ] `scene.export_triangles()` (scene), in five languages: the world-space triangles of every model, with each instance's `model` slot, the model's transform and the world matrix of its node applied, as flat `f32`, nine per triangle. Loaders keep no CPU copy for the host today
- [ ] The Mario walk-through, milestone M1, an example on libsm64 that `write_vertices` and `export_triangles` serve
- [ ] `integrations/pbrt`, a native Rust loader that returns a `Scene`, on the 0.13.0 core, so M2 and M3 read scenes through it
- [ ] The compute path tracer integration, milestone M3, which may start any time after 0.13.0
- [ ] The snapshot harness (Feature tracks), then a pbrt-v4 baseline lit with today's lights. Depth-only passes, which `submit` refuses with `NoTarget` in 0.13.0, and the depth array fix: a depth texture with several layers asks for a 3D texture today. Then a shadow mapping example

### More additions

- [ ] Unlit and toon `Material` presets over registry WGSL (3D)
- [ ] Frustum culling on `Camera`, beside `sort`, which a pass that keeps models runs before it sorts (3D)
- [ ] Picking, debug lines drawn as quads, and GPU timings, which are optional and coarse on WebGPU and absent on WebGL2
- [ ] The web fetch and the WebGL2 readback yield reach the browser through `js_sys::global()`, so both work inside a web worker, where `web_sys::window()` is absent

### Integrations

Official wrappers live in an `integrations/` folder at the repository root. They are platform-specific examples, outside the five-language rule. They call only documented API, build on the published 0.13.0 packages, and each has a smoke test against a shared fixture. Each one stays an example in the repository until its API settles, and then publishes to npm or PyPI.

- [ ] `@fragmentcolor/element`, a web element layer with no framework: wasm loading, sizing, the frame loop, pausing and disposal. The website's two FragmentColor elements, the tutorial demo and the shader gallery, move onto it
- [ ] `@fragmentcolor/react` on top, with components named after FragmentColor objects. JSX order is pass order, `<Model>` children add themselves in that order, and a prop that names a slot, such as an example `tint`, writes it on the model's mesh through `set`
- [ ] A controls panel that reads `layout()` on `Shader`, `Mesh` and `Buffer` and writes through `set`: a plain web panel and a React `<Controls>` first, egui later
- [ ] A Shadertoy clone built on the React package, which replaces `platforms/web/repl`
- [ ] An LLM plugin that calls a server the app owns
- [ ] A mini scene editor on `Scene` and `Model`, with gizmos. It saves a scene's models, materials, cameras and lights to JSON and loads them back
- [ ] Candidates on request: rich animation over ozz-animation, Tiled and LDtk maps, a 2D UI and HUD, glyphon for Rust hosts and XR, both after host interop, complex text shaping, and Spine, whose runtime license asks every user to hold an editor license

### Feature tracks

The feature tracks, further down, land as patches from 0.13.1 on. A track item that breaks a signature waits for 0.14.

## 0.14: light and surfaces

Lighting is the one area where several features share a contract and each would break it: the PBR bindings, the `lights` layout and the bytes `light.pack()` returns, all under WebGL2's 16 sampled textures per stage and 4 bind groups. Shipped one patch at a time, shadows, image-based lighting and area lights would change that contract three times, so 0.14 designs them once, together with the targets and texture shapes they need. A pbrt-v4 subset, rasterized, is its acceptance test, the role the Bevy example plays for 0.13.0. Skinning, sprites and text only add calls, so they ship before it as 0.13.x tracks.

- [ ] **PBR lighting, designed once** (3D). Shadows (directional cascades, spot, point cubes), image-based lighting from `.hdr` and equal-area octahedral maps, area lights through new `LightKind` cases (rectangle, disk, sphere) with LTC shading, and a glass preset, in one binding plan that fits WebGL2 and the clustered lights of 0.12.7. It versions the `Lights` layout if area lights or shadows need more room than the free words it has, and breaks the `light.pack()` bytes, the PBR bindings that `layout()` reports, and `LightKind`, which has three cases and no `#[non_exhaustive]`. The 0.13.x shadow and lighting examples become this API
- [ ] **Targets with options** (core). Format and sample count per target, such as `Rgba16Float` and `R32Uint`, several color targets per pass (WebGL2 allows 4), and targets that are one layer, face or mip of a texture. A cube-map `Camera` (3D) sits on top, for reflection probes and point-light shadows. It breaks the renderer-wide sample count and the shape of pass targets
- [ ] **Texture shapes and formats** (core) that 0.13.0 leaves out: depth cubes and depth arrays for shadows, and the target, depth-stencil and storage formats the items above need
- [ ] **Pipeline completion** (core). Stencil with a depth-stencil format, and `Blend::Custom` with a `BlendFactors` record, up from Later. A new `Blend` case breaks an exhaustive `switch` in Swift and `when` in Kotlin, so it waits for a breaking minor
- [ ] **Acceptance test: a pbrt-v4 subset, rasterized** (test tooling), milestone M2 below. `bmw-m6` and `contemporary-bathroom` come first, the only scenes of the first set with a stated license

A wgpu major that brings video without a copy can ride along. 0.14 does not wait for one.

## Reference scenes

Real content tests the layers harder than synthetic cases. Four milestones tie this goal to the releases that ship what they need.

| Milestone | Release | What it proves |
| --- | --- | --- |
| M1. The Mario walk-through on libsm64 | 0.13.x | `write_vertices`, `export_triangles`, custom WGSL, and a host that owns the clock |
| M2. A pbrt-v4 subset, rasterized | 0.14, as its acceptance test | Light and surfaces |
| M3. A compute path tracer that matches pbrt's reference images | Any time after 0.13.0 | `Buffer`, runtime-sized arrays, `submit` and float textures carry a full renderer with no new core API |
| M4. Bruno Simon's Three.js portfolio, reproduced | 0.13.x | What a complete interactive Three.js scene needs that FragmentColor lacks |

Two rules hold throughout. No scene file, reference image, ROM or game asset enters the repository, a release, CI artifacts or the website: tests read a local checkout of `mmp/pbrt-v4-scenes` at a pinned commit, named by `FC_PBRT_SCENES`, or a cached sparse checkout on the snapshot track's macOS CI step, and skip when neither exists. Loaders, harnesses and renderers built for the goal are integrations or examples, native Rust first, outside the five-language rule, and every FC call they need ships in all five languages.

From 0.13.0 the goal takes `write_vertices`, `anisotropy` and mirror wrap, `dimension` and layers with the new format cases, the four new `Pipeline` fields, the `exr` decoder, `Buffer`, runtime-sized arrays, `submit` and `initialize`.

### M1. The Mario walk-through (0.13.x)

A harness links libsm64 (CC0, at a pinned commit) and FC, and neither calls the other. Each 1/30 s tick, `sm64_mario_tick` takes the stick, the buttons and the camera's flat look direction, and fills four `f32` arrays (position, normal, color, uv) with up to 1024 posed triangles, unindexed, in world space and wound counterclockwise. libsm64 poses Mario on the CPU, so FC draws a plain mesh and needs no skinning.

- [ ] `examples/sm64`, desktop Rust first (macOS, Linux, Windows), never published: `publish = false`, outside the default members and every workflow. Its build compiles libsm64 with `cc` and runs `import-mario-geo.py`, which downloads Mario's decompiled geometry
- [ ] It reads a US ROM from the path in `FC_SM64_ROM`, checks its SHA-1 (`9bef1128717f958171a4afac3ed78ee2bb4e86ce`) and exits with a message when it is absent or differs. Nothing from the ROM, the atlas, frames or videos reaches the repository, CI, releases, the website or the blog
- [ ] Each tick, `mario.write_vertices(posed)` replaces at most 3072 vertices of 44 bytes and reuses the GPU buffer. The 704 by 64 RGBA8 atlas goes through `create_texture` and `set_sampler_options`, as today
- [ ] Custom WGSL in the harness, from libsm64's GL 3.3 test shader: `mix(color, atlas.rgb, atlas.a)` times `0.5 + 0.5 * clamp(dot(normal, light), 0, 1)`
- [ ] Collision from `scene.export_triangles()`, which the harness scales to 100 units per meter, rounds to `int32` and feeds to `sm64_static_surfaces_load`, keeping the triangles within about 20 m of Mario and reloading them every 5 m he moves. The scene is glTF or OBJ through `Scene::load` first, then `villa` through `integrations/pbrt`
- [ ] Fallback, the rigid-parts route: the example reads the ROM in pure Rust into 15 rigid parts on a 20-joint tree, where no vertex blends two joints, evaluates the walk keyframes at 30 Hz and calls `set_transform` on 15 `Model`s along a scripted path
- [ ] Later: the web, through libsm64's Emscripten build as a module of its own that page JavaScript ticks before it calls FC's `writeVertices`, then iOS. A search on 2026-10-22 found no Android build of libsm64

**Test.** `cargo test -p sm64 --release -- --ignored`, headless on a `TextureTarget`, on a local machine only. It loads the scene, exports its triangles and runs 300 ticks of a fixed script: stick forward, one turn, one jump. Each tick the mesh holds `numTrianglesUsed * 3` vertices and keeps its GPU buffer after the first. Mario ends more than 5 m from the spawn, and while grounded his height matches `sm64_surface_find_floor_height` within 5 units, which proves scale and winding. Pixels inside his projected bounds differ from a frame rendered without him, and no reference image is stored.

### M2. A pbrt-v4 subset, rasterized (0.14)

First set: `bmw-m6` and `contemporary-bathroom`, then `pbrt-book`, `killeroos`, `villa` (daylight and lights on) and `ganesha`.

- [ ] `integrations/pbrt` covers the directives the first set uses: `Include` and gzip, binary PLY in both byte orders, quads split into two triangles, tessellated spheres and disks, one Loop subdivision step for `killeroos`, and the left-handed flip, reversing the winding when a transform's determinant is negative. Emissive meshes become area lights. It reads `fov` across the shorter image side
- [ ] Materials follow the fallback in Falcor's pbrt-v4 importer: `diffuse` to base color, `conductor` and `coatedconductor` to metallic with base color from the `eta` and `k` spectra, `coateddiffuse` to roughness `sqrt(alpha)`, `dielectric` to the glass preset. `measured` gets a fitted value, `mix` blends by `amount`, and displacement becomes normals at load time. Spectra and blackbody emitters turn into RGB through tables from pbrt-v4's data (Apache-2.0)
- [ ] 0.14 lights the scenes: image-based lighting from the equal-area octahedral maps, shadows, area lights, and HDR through a float target and the registry's `color/tonemap_aces`, with `iso`, `exposuretime` and the sensor curve folded into one exposure value

**Test.** `cargo test --test reference_scenes` renders each scene at its reference resolution and compares it with a direct-light reference that pbrt-v4 renders at a pinned commit (`path` integrator at `maxdepth 1`), with the same tone map on both sides. The hybrid score in `tests/snapshots.rs` gates it, at a per-scene tolerance set on the first passing run that may only tighten. A second score against the EXR a scene ships reports without gating, since those references carry indirect light from 256 to 65,536 samples per pixel.

### M3. A compute path tracer (any time after 0.13.0)

- [ ] `integrations/pathtracer`, a WGSL compute path tracer over the scenes `integrations/pbrt` loads: a SAH BVH built on the CPU in storage `Buffer`s with a fixed traversal stack, a second level for instances, textures in `texture_2d_array`s grouped by size, and samples that accumulate in a float `Texture` through `submit`. It ports pbrt-v4's `path` integrator in RGB, and runs natively and on WebGPU, not on WebGL2. WebGPU promises only 128 MiB per storage binding, so `ganesha`'s BVH may need splitting there
- [ ] Targets: the first set, then `transparent-machines` and `dambreak`, where refraction drives the look
- [ ] Later: hardware ray queries, native only, inside the integration through host interop (Open decisions), and volumes, hair and subsurface scattering

**Test.** At `maxdepth 1` the tracer matches M2's references, which checks rasterizer against tracer. At full depth, the relative mean squared error against the shipped EXR, after a fixed sample count and a 4x downsample, stays under a per-scene bound.

### M4. A Three.js portfolio, reproduced (0.13.x)

Bruno Simon's portfolio is a driveable 3D scene built on Three.js, with its code, music and models published under the MIT license. Rebuilding it one to one with FragmentColor measures the distance to Three.js on a real project. Three.js is higher level and runs only on the web, so the goal is a list of what FragmentColor should offer for 3D work in every language, not a copy of Three.js's API.

- [ ] Confirm the license of the version to port, models and music included
- [ ] `examples/folio`, desktop Rust first, reading the assets from a local checkout under the rule above
- [ ] A gap list kept beside the port: each Three.js feature it needs that FragmentColor lacks, such as loaders, materials, physics hooks, audio, post-processing and input, with a ROADMAP item or a reason it stays out

### Licensing

`mmp/pbrt-v4-scenes` has no root license, and each scene states its own or none. CI uploads no image of a scene without a stated license, and converted copies of the CC BY-ND scenes (`kroken`, `watercolor`) are never hosted. Not legal advice.

| Scene | License stated | Publish renders |
| --- | --- | --- |
| `bmw-m6` | CC0 (Blend Swap) | Yes |
| `contemporary-bathroom` | CC0 (Blend Swap). `American_walnut_pxr128.png` is Pixar One Twenty Eight (CC BY 4.0), and the other textures name no source | Yes, crediting Pixar |
| `transparent-machines` | Beeple's terms: free, no credit needed | Yes |
| `pbrt-book`, `dambreak` | None (Karl Li) | After written permission |
| `killeroos` | None (headus) | After written permission |
| `villa` | None (Florent Boyer) | After written permission |
| `ganesha` | None (scan by Wenzel Jakob) | After written permission |
| Mario | Nintendo | Never |

Out of scope for now: `kroken`, `watercolor` and `landscape`, which need gigabytes, refraction in the rasterizer, and Ptex, which no scene uses.

Sources, read 2026-10-22: github.com/mmp/pbrt-v4-scenes, github.com/mmp/pbrt-v4, Falcor's `PBRTImporter.cpp`, github.com/libsm64/libsm64, github.com/n64decomp/sm64, blendswap.com/blend/13303, renderman.pixar.com/pixar-one-twenty-eight and www.beeple-crap.com/resources.

## Later: splits, a C target, interop and GPU features

### Crate splits

`fragmentcolor` stays one crate, plus the macro and rules crates if `#[api]` lands, until a written trigger fires. After a split it stays the flat root that re-exports every member, so imports keep working.

- [ ] A naga-only crate that parses, reflects and packs, when a Rust host cannot take FragmentColor's wgpu major
- [ ] A separate 3D crate, if its dependencies or release pace drift from core's. The layer check already marks its boundary
- [ ] A binding facade, when FragmentColor's own FFI or the C target starts

### C target and optional backends

- [ ] A C arm of `#[api]` (`extern "C"`, a header, panic catching, borrowed bytes) once the API is stable
- [ ] BoltFFI as an optional backend behind `#[api]`, once its rewrite lands, adopted only when every healthcheck passes with no change to the generated API

### Host interop

- [ ] Interop with the host's own GPU objects: `Renderer::from_device(device, queue)`, `Renderer::encode(&renderable, &mut encoder, &view)` and `From<wgpu::Texture>` in Rust, with a native equivalent in each language where the platform has one, such as `Renderer.fromDevice(gpuDevice)` in JavaScript. The calls that take wgpu types carry the one documented exemption in the binding attribute. They add calls only, so they land as a patch when work starts

### GPU features

- [ ] Video without a copy, as a new call, once wgpu's `ExternalTexture` reaches Vulkan and the web. wgpu 30.0.1 has it on DX12 and Metal
- [ ] Indirect draws and GPU culling, which WebGPU has and WebGL2 lacks, and occlusion queries
- [ ] Bindless and multi-draw-indirect stay out of core until WebGPU standardizes them

### More engines and frameworks

- [ ] `examples/interop/threejs`, built in CI, with its blog post
- [ ] Blog posts for Godot, Unity, Unreal, Raylib and Löve
- [ ] Svelte and Vue wrappers, and native view wrappers on request

## Open decisions

The plan follows each recommended answer until the question is settled.

Settled: the earlier open questions, each on its recommendation, except the per-object API. `Camera` and `Light` stay handles that push nothing into shaders, `Renderer::load` is warm-up for any renderable, `Scene` is one pass through `scene.pass()`, USD lands in a 0.13.x patch after a license review and a wasm and mobile trial, and the custom data rules hold. Instancing lives on `Mesh`, with index views as documented sugar, a plain range on `pass.add_mesh` beside the shader's own mesh list, and a `Model` with no instance calls. `write_vertices`, `write_instances`, `set` and `layout()` are the verbs. A later shader never widens a mesh's instance slots: adding it fails with an error that names both shaders, unless a compelling case turns up. `Texture::write_region` stays beside `write`, which writes the whole texture through the same path. The parts of a model live one level up: `Model` stays one mesh and one material, and `SceneNode`, a handle in the scene layer, groups models, cameras, lights and child nodes, with `scene.nodes()` returning root nodes in place of the planned record. Nodes hold cameras and lights from 0.13.0, and a kept group draws mirrored instances with the opposite `front_face`. Also settled: the 0.13.0 additions, the 0.13.x tracks, 0.14 as light and surfaces, and the Reference scenes goal.

Settled before that: one 0.13.0 with every item; `Scene`, `SceneObject`, `Pass::add`, `Renderer::load` and `Model` kept; no public `Batch`; `set_pipeline` with `pipeline()`; override constants through `set`; `add_plugin`; a sync `submit` with `initialize`; the JavaScript throw with `set_source`; magic uniforms; the mobile renames; HDR; the restart of `v0.13.0-dev`; the full package on every platform; the layers with a `build.rs` check; Cargo features for dependencies only, on by default; and loaders that return `Scene`, with no `Asset`. Settled earlier still: `Pipeline` and `Compare`, `Buffer` with write-through `set`, runtime-sized arrays first, core names in Swift and Kotlin with no aliases, plain JavaScript objects when possible, and integrations as platform-specific examples.

A few more questions on `Pipeline`, shared data and the bindings are still open. The ones under Scene nodes set rules the scene work needs, and the Reference scenes goal raises the three under its heading below.

### Scene nodes

- **Cameras and lights in a copy: new handles, or the same ones?** Recommended: `node.share()` keeps the same `Camera` and `Light` handles, so each copy counts them once, and a copy that needs a light of its own removes the shared one and adds a new one. Why: new handles need a copy call on `Camera` and `Light`, since `Camera` has no getters for its projection, and a pass uses one camera anyway.
- **A model, camera or light under two nodes: allowed, or an error?** Recommended: allowed, counting once under each, the way one mesh serves several models. Why: none of them has a parent link, since the 3D layer cannot name `SceneNode`, and only a crate-private hook through the seal could give them one.
- **`add_to` on a node?** Recommended: none in 0.13.0, so a pass keeps a node with `pass.add`. Why: a node draws through the kept group, and a later `node.add_to(&pass)` only adds a call.

### Reference scenes

- **Hardware ray queries: FC API or not?** Recommended: not in core. They come Later, native only, inside the path tracer integration through host interop. Why: wgpu 30 marks `Features::EXPERIMENTAL_RAY_QUERY` experimental, FC turns experimental features off, and WebGPU has none, so parity would need a software fallback. They change speed, not the image.
- **The `.pbrt` loader: `integrations/pbrt` or a `Scene::load` decoder?** Recommended: `integrations/pbrt`, native Rust, promoted to a `pbrt` decoder only on request. Why: the material mapping is approximate and will change, and one scene reads hundreds of sibling files, which on the web means a fetch for each. A decoder adds no type, so a later promotion still fits a patch.
- **Who shades Mario?** Recommended: custom WGSL in the harness, with no N64 preset. Why: libsm64's test shader mixes vertex color into the atlas by texel alpha, which a PBR base color does not do, and core already runs custom shaders.

## Where each item went

Latest moves:

- **Settled:** every earlier open question on its recommendation, except the per-object API, which the instancing design replaced. Then the slot rule and `Texture::write_region`, both on their recommendations. Then where the parts of a model live: one level up, in `SceneNode`, with `scene.nodes()` returning root nodes, cameras and lights in nodes from 0.13.0, against the recommendation to wait, and the mirrored run in the pass.
- **Scene nodes, new in 0.13.0:** `SceneNode` with 19 calls and `SceneError`, `model.share()`, `Eq` and `Hash` on `Model` and `SceneNode`, cameras and lights placed by their node, the mirrored run with its second pipeline, and the Scene nodes guide. The loader builds a node per glTF node with a model per primitive over shared meshes, and the mirrored-node fix moves from the loader to the pass.
- **Dropped with them:** the `SceneNode` record with its `instance` index, its place among the plain JavaScript objects and the `#[non_exhaustive]` records, and the loader's one model per mesh and material with an instance per node.
- **Opened:** three questions on scene nodes.
- **Instancing, new in 0.13.0:** instances live only on `Mesh`, with `add_instance()` returning the index, `remove_instance`, `resize_instances`, `instance_count`, `write_instances`, `set` and `get` through the key grammar, `layout()`, `share()` and the index views `mesh.instance(N)` and `mesh.instances()`. `write_vertices` replaces a mesh's vertices. `pass.add_mesh` takes a plain instance range, the shader's mesh list stays, and the loader builds one `Model` per mesh and material with an instance per node.
- **Dropped with it:** per-object rows are gone, and with them `model.row()`, the one-instance `Model`, `shader.validate_rows`, instance `Buffer`s with their layouts, the `Buffer` form of the `add_mesh` argument, `mesh.set_instances`, `buffer.fields()`, which `layout()` replaces, `Instance` with `Vertex::create_instance`, `add_instances`, `clear_instances`, `set_instance_count`, and the backlog's partial writes, which span keys cover.
- **New in 0.13.0:** `depth_bias`, `color_write`, `front_face` and `alpha_to_coverage` on `Pipeline` (the middle two from the backlog), `anisotropy` and mirror wrap on `SamplerOptions`, `dimension`, the layer count and the compressed format cases (from the KTX2 track), `#[non_exhaustive]` on returned records, the `joints0` and `weights0` keys, the two glTF loader fixes, the `exr` feature, and the removal of `create_external_texture` (from the Live media track).
- **New in 0.13.x:** the Characters track, with skinned animation and glTF node animation from Later, the 2D layer, `scene.export_triangles`, the Mario walk-through, `integrations/pbrt`, the path tracer integration, unlit and toon presets and frustum culling from Later, depth-only passes, picking, debug lines, GPU timings and the workers fix.
- **New in 0.14:** PBR lighting with shadow mapping and the glass preset from Later and the area cases of `LightKind`, targets with options with the cube-map `Camera` from Later, texture shapes and formats, stencil from the All bindings list, and `Blend::Custom` from Later.
- **New goal:** Reference scenes, with M1 to M3.
- **Landed in 0.12.6:** the exact `[w, h]` `ScreenRegion`, `WindowTarget` in JavaScript and Python, `WindowTarget`, `TextureTarget` and `TextureRegion` in Swift and Kotlin, with aliases for the old names until 0.13.0, uniffi names in reflect, `TextureUsage` flags as JavaScript numbers, vertex and instance attributes typed by the shader, an internal `PassInput`, KTX2 detection by identifier, the `#[lsp_doc]` check on methods, the one-frame JavaScript list render, and, from 0.13.0, JavaScript `Error` objects, the throwing JavaScript `Shader` constructor and the applied viewport.

Moves before that:

- **Kept in 0.13.0:** `Scene`, `SceneObject` (now sealed and in core), `Pass::add`, `Renderer::load` (now warm-up), `Scene::load` (now async) and `Model::set_visible`.
- **Picked on packaging:** the full package on every platform, the layers with a `build.rs` check, and Cargo features for dependencies only, on by default.
- **New in 0.13.0:** neighbor merging with one lasting instance buffer per pass, `Pass::remove`, `scene.pass()`, `skip_defaults`, `set_ambient`, `model.name()`, OBJ loading, URLs and bytes in every language, the scene's depth target, the layer check, `SceneObject` and `Renderable` as the core traits the higher layers implement, the `gltf` and `obj` features, `network` in the defaults, decoder errors that carry a message, reflect's new feature list, and the per-feature CI jobs. Custom data support brought slot layouts, `[i].slot` keys, `VertexInput.docs`, `ModelError`, `scene.nodes()` and the Custom components guide.
- **New in 0.12.x:** the stray instance fix, the duplicate scene pass fix, the primitive mode check, the `SceneInner` rename, and the `Model`, `Material`, `Renderer::load` and `Scene.load` doc fixes. Custom data support brought the glTF `extras` feature, the `Material::custom` page and `Mat4` comment fixes, the PBR normal matrix, and the `AGENTS.md` rule that FC never calls host code during `render` or `submit`.
- **New in 0.13.x:** USD loading.
- **Dropped:** the scene removal, `Asset`, value records for `Camera` and `Light`, separate draws for models that share a mesh and a material, the backlog's type aliases, a `Material` per tinted object, and "extra bytes per model wait for a request". `Batch` stays dropped as a public type.
- **Withdrawn after the packaging pick:** an earlier `bindings` set, features that leave the 3D or scene layer out, the rules such features would need in reflect, docs, healthchecks and docs.rs, and prebuilt slim or core packages.

Moves from the previous roadmap:

- **To the front of 0.12.x:** the hidden blocks branch, runtime-sized arrays and atomics. The `#[api]` trial becomes the gate before 0.13.0.
- **New in 0.12.x:** the docs.rs check, `glsl-out`, override defaults, the entry point rule, the stale instance count, the typed `create_target` error, three stale references and the `PARITY` note, the Python error class, the internal renames and the qualifier rule.
- **New in 0.13.0:** `WindowTarget` in JavaScript and Python, `PassInput` and `TextureData` on mobile, override constants, the entry point error, `initialize`, the instances argument, the checks and frame rule for a pipeline that fails at render time, every public error enum as `#[non_exhaustive]` and the `InitializationError` variant for a surface with no formats, `TextureId` and `TextureUsage` as JavaScript numbers, the GIL release, the live coding guide, `model.add_to`, `camera.sort` and the `build.rs` front-end check.
- **From 0.12.x to 0.13.0, because each breaks callers:** the Swift and Kotlin names, the object forms of the free functions, the JavaScript throw with `set_source`, the parse error fields and HDR. The Swift and Kotlin names and the JavaScript throw then landed in 0.12.6, with aliases for the old names until 0.13.0.
- **From the previous roadmap's 0.13.x, 0.14 and 0.14.x to 0.13.0:** the 3D layer, plugins, the ported examples, the Bevy example, storage roots owned by their shader, freeing on drop, non-blocking `render` and the Cargo features. `App` comes from Later.
- **From the feature tracks to 0.13.0:** `set_source` and magic uniforms.
- **From the previous roadmap's 0.14.x to 0.13.x:** the bind group cache, one error scope per frame and the graph version, joined by the pipeline eviction rule.
- **From 0.13.0 to Later:** custom blend factors, which now go to 0.14.
- **Dropped:** `add_range`, `Batch`, the scene deprecation steps, `Shader::new_watched`, `ShaderHealth` and `Pass::swap_shader`.

## Feature tracks

These land as patches from 0.13.1 on, in the order below. The composition preprocessor comes first, because it builds on the plugin table and the 2D layer needs its pass shader entry kind. The Characters and 2D layer tracks sit in the 0.13.x section.

### Composition

Function-level WGSL composition with a hosted registry.

#### Preprocessor

- [ ] A preprocessor that hoists `use` imports, walks their dependency graph with cycle detection, drops unused functions, mangles names and emits one module
- [ ] Imports resolve through the plugin table, after the parts passed in the same list
- [ ] Form A: `use @fc/sdf/sphere;` and `use @fc/sdf/smooth_min as smin;` at the top of a WGSL file
- [ ] Form B: `@fc/sdf/sphere(p, 1.0)` anywhere in a WGSL body. The preprocessor rewrites it to a mangled call and hoists a `use`. Full `https://…(...)` URLs work the same way, and a `use … as <alias>` wins over an inline name
- [ ] Imported functions may `use` other slugs
- [ ] Only functions reachable from the entry points are emitted; unused imports are warnings
- [ ] Imported functions get a prefix (`__fc_sdf_sphere`) behind thin alias shims
- [ ] Function packs have no `@group` or `@binding`, no entry points, exactly one public `fn`, and no `var<private>` or `var<workgroup>`; module-scope `const` is allowed
- [ ] Errors: `ShaderError::{ImportCycle, NameCollision, SignatureMismatch, InvalidRegistryEntry}`

#### Registry client and service

- [ ] `Shader::new("@fc/bloom")`, `Shader::new("/bloom")` and `Shader::new("@alice/neon-grid")` resolve from the registry
- [ ] Version pins: `@fc/bloom@1.2`, `@fc/bloom#sha256:abc…`, and the same for full URLs
- [ ] A local cache (the platform data directory on native, IndexedDB on the web), keyed by the SHA-256 of manifest and body, with a 256 MiB LRU cap. Plugin output never enters it
- [ ] A resolve policy (`Latest`, `Locked`, `Offline`) set through a static on `Shader`, since resolution happens before any renderer exists. `Locked` refuses unpinned slugs, and `Offline` serves only from the cache
- [ ] Network errors, 404s and hash mismatches surface as registry errors, named when the registry lands. `PluginError` covers host plugins only
- [ ] A hosted registry at `shaders.fragmentcolor.org`: a static origin with immutable entries addressed by content hash
- [ ] Registry entries: a TOML manifest and a WGSL body, with the declared function signature and dependencies
- [ ] Two entry kinds: function packs (primary) and pass shaders (chain stages)
- [ ] CLI: `fragmentcolor publish [path]`, `resolve <slug>`, `cache --list` and `--clear`, `lint <file.wgsl>`

#### Stage chains and a WGSL standard library

The registry at `https://fragmentcolor.org/shaders/` already serves about 230 helper functions in 21 categories, such as `sdf/sphere`, `noise/fbm2`, `color/tonemap_aces` and `easing/in_out_cubic`. Native builds embed them by default, the web build fetches them, and they compose today when their slugs sit in the list `Shader::new` takes.

- [ ] A list of stages such as `["scene", "/bloom", "/tonemap"]` builds a list of passes in which each stage samples the previous one. It hangs off an existing object, since the public API has no free functions
- [ ] A chainable form: `shader.then("bloom").then("tonemap")`
- [ ] A post-processing stage is a registry entry or a shader passed as parts, so a custom stage needs no Rust trait
- [ ] Each stage's parameters come from its shader's `layout()` and are set with `shader.set`, so stages need no types of their own
- [ ] Post-processing stages as pass shaders in the registry, built on today's `postfx` and `color` helpers where they exist: `@fc/bloom`, `@fc/ssao`, `@fc/tonemap-aces`, `@fc/tonemap-reinhard`, `@fc/chromatic`, `@fc/vignette`, `@fc/grade`, `@fc/film-grain`, `@fc/motion-blur`, `@fc/fxaa`, `@fc/depth-of-field`
- [ ] Serve today's helper categories under `@fc/` (for example `@fc/sdf/sphere`) with registry manifests, while the current slugs keep working
- [ ] Fill the gaps in today's categories, such as `hash31`, `simplex3`, `gamma`, `back_in_out` and `blue_noise`
- [ ] Examples: Hello Composition, Hello Post-FX

#### Build-time minification

- [ ] Strip comments and rename identifiers in bundled and registry shaders at build time
- [ ] Per-platform specialization (immediates on native, the uniform rewrite on the web) as a build step instead of at run time

### Live media

External textures (video), audio-reactive textures, a Shadertoy bridge, and live input sources (webcam, microphone, gamepad, touch). What creative coders need for music videos, game prototypes and AR/VR demos.

#### External textures (video)

`Renderer::create_external_texture` goes in 0.13.0. It exists on the web, iOS and Android, and each one returns an error that says external texture import is not implemented yet. A new call returns once wgpu's `ExternalTexture` reaches every backend FC ships on (Later), so video frames sample without a copy in WGSL through `texture_external` and `textureSampleBaseClampToEdge`. It adds a call, so it lands as a patch.

- [ ] Web: map `HTMLVideoElement` to `wgpu::ExternalTexture`
- [ ] iOS: `CVPixelBuffer` to `wgpu::ExternalTexture` through `CVMetalTextureCacheCreateTextureFromImage`, with a Swift extension that takes a real `CVPixelBuffer`
- [ ] Android: `SurfaceTexture` to `EGLImage` to an `OES_EGL_image_external` sampled texture, with a Kotlin extension that takes a real `SurfaceTexture`
- [ ] Python and desktop Rust: a frame source that suits each, such as a NumPy array or raw bytes with a format and size, since neither has a portable video-frame source
- [ ] A healthcheck that decodes one frame on each platform and reads it back with `Renderer::read_texture`

#### Input names and a Shadertoy bridge

Core fills the five magic uniforms from 0.13.0 on. This track covers input and Shadertoy.

- [ ] Input names such as `mouse` (position and click state) and `scroll` follow a documented convention that integrations fill
- [ ] The same convention in Rust, JavaScript, Python, Swift and Kotlin
- [ ] `@toy/abcdef` as a built-in plugin entry: fetch a public Shadertoy by ID, translate GLSL to WGSL with naga, and wrap the `mainImage(out vec4, in vec2)` contract
- [ ] `Shader::toy(source)` for pasted bodies
- [ ] Map `iTime`, `iResolution`, `iMouse`, `iFrame` and `iChannel0..3` onto magic uniforms, input names and texture bindings
- [ ] Multi-buffer Shadertoy layouts (Buffer A to D) become a list of passes. The docs say these Shadertoy buffers are unrelated to `Buffer`
- [ ] Example: Hello ShaderToy

#### Audio-reactive textures

- [ ] `renderer.create_audio_texture(source)`: FFT bins and the waveform as a `texture_1d<f32>`
- [ ] Sources: an `<audio>` element, an `AudioContext` node, the microphone, a URL, a file (native)
- [ ] Configurable sample rate and FFT size, with defaults that suit music visuals
- [ ] Example: Hello Audio-Reactive

#### Live input

- [ ] `renderer.create_webcam_texture()`: the camera feed as a `texture_2d<f32>` on every platform, through `getUserMedia` on the web and the platform camera APIs elsewhere
- [ ] One `create_video_texture(path_or_url)` on every platform
- [ ] Input uniforms on the input name convention: `mouse`, `keyboard` (a bit set), `gamepad` (sticks and buttons), `touches` (positions), fed by the integrations
- [ ] Example: Hello Webcam

### Live coding

Hot reload, a Shadertoy clone that builds controls from `shader.layout()`, and shader sharing. Prototyping is one of the reasons this library exists, so hot reload is part of the first experience. Its core call, `set_source`, ships in 0.13.0.

#### Hot reload

Both items call only documented API, and both keep drawing the last good program when an edit fails.

- [ ] A Vite build plugin or documented convention that forwards `.wgsl` changes to `shader.setSource()` in JavaScript
- [ ] A desktop example that watches a file with the `notify` crate and calls `set_source`

#### The Shadertoy clone and generated controls

These fill in the Shadertoy clone and the controls panel under 0.13.x Integrations, with documented calls only.

- [ ] Editor and canvas side by side, with a resizable divider, pane swap and a fullscreen canvas
- [ ] Monaco or CodeMirror 6 with WGSL highlighting
- [ ] Apply each keystroke through `set_source`, with a 250 ms debounce
- [ ] Inline errors from the line and column on parse errors: the offending line underlined, naga's message in a tooltip
- [ ] A controls panel built from `shader.layout()`, plus instance slots from `mesh.layout()`, with one widget per type: slider (`f32`), stepper (`i32`, `u32`), checkbox (`bool`), XY pad (`vec2`), sliders or a color picker (`vec3`, `vec4`), a grid (matrices), a file picker (textures)
- [ ] Hints in WGSL doc comments, read from the `docs` field of `layout()`: `/// @range a..b @step s @default d`, `/// @color`, `/// @xy`, `/// @file`
- [ ] Every change goes through `shader.set(...)`

#### Sharing and gallery

- [ ] Shareable URLs: `{ source, uniforms, inputs }` gzipped and base64-encoded in the URL hash, restoring the clone's whole state
- [ ] A shader built from a plugin prompt stores `shader.source()` next to the prompt, so a shared link draws the same shader
- [ ] A "Save to gist" button (anonymous)
- [ ] A gallery of 20 to 30 chosen demos: a glTF viewer, an audio-reactive visual, a compute simulation (fluid, boids or slime mold), an SDF raymarcher, a post-processing stack, a Shadertoy port, a ten-line shader
- [ ] Each card has a thumbnail, a description, "Open in the editor" and "View source"
- [ ] A "Shader of the Day" on the homepage
- [ ] Example: Hello Live-Coding

### Asset pipeline: textures

KTX2 already loads, parse-only through the `ktx2` crate: uncompressed RGBA8, RGBA16F, R16, Rg16 and Bgra8, BC1 to BC7, ASTC 4×4 and 8×8, and ETC2, with the pre-baked mip chain uploaded as is and a clear error when the GPU lacks a compression feature. 0.13.0 adds `dimension`, the layer count and the compressed format cases to the public records. The work below covers the gaps.

#### KTX2 and compressed formats

- [x] `create_texture` sends bytes, paths and URLs that start with the KTX2 identifier to the KTX2 loader on its own (0.12.6)
- [ ] Cube maps (`faceCount = 6`): upload all six faces and expose a cube view, on 0.13.0's `dimension`
- [ ] Array textures (`layerCount > 1`): a 2D array view
- [ ] 3D textures (`pixelDepth > 1`): a 3D view
- [ ] Zstandard and ZLIB supercompression (BasisLZ is under Basis Universal below)
- [ ] `bytes_per_pixel` becomes `bytes_per_block(format) -> (u32, BlockSize)`, since compressed formats upload by block
- [ ] CI fixtures per platform: BC7 on desktop, ASTC 4×4 on Apple, ETC2 on the Android emulator, a six-face cube map, and a BC7 load on an adapter without BC that returns the capability error
- [ ] An availability matrix in `docs/guides/loading-compressed-textures.md` (desktop, iOS, Android, WebGPU)
- [ ] WebGL2 compressed textures stay out of scope; Basis Universal covers them

#### Progressive texture streaming

KTX2 stores the smallest mip last, which suits sequential loads: parse the header, fetch the smallest level, render at low resolution right away, then fetch larger levels in the background and lower the sampler's `lod_min_clamp` as each one arrives.

- [ ] `StreamingTexture` wraps `Texture` with a loading state (`Loading { available, total } | Complete`) and derefs to `Texture`
- [ ] `Renderer::stream_texture(input)` fetches the header and smallest mip, then returns
- [ ] Each new level uploads with `queue.write_texture` at its mip level and lowers `lod_min_clamp`, with no re-creation and no new bind groups
- [ ] `StreamingTexture::on_progress(...)` for progress bars; dropping the texture cancels its fetches
- [ ] Web: `fetch` with a `Range` header and bounded concurrency
- [ ] Native and mobile: a `Range` variant of `crate::net::fetch_bytes`, one worker per active stream, and backpressure when the budget runs out
- [ ] `Renderer::set_streaming_budget(parallel, bytes_per_sec)` with sensible defaults
- [ ] Tests against a synthetic five-level KTX2 served from a local test server: smallest mips first, cancellation on drop, `lod_min_clamp` only ever decreasing
- [ ] `docs/guides/streaming-textures.md` and a Hello Streaming Textures example with a 4K KTX2

#### Basis Universal transcoding

Basis Universal encodes once and transcodes at load time into the device's native compressed format (BC7, ASTC or ETC2), so one file serves every platform. The C++ dependency adds about 5 MiB, and like every dependency it sits behind a feature.

- [ ] A `basis` dependency feature that enables `basis-universal`, on by default like the others. It lands once the crate builds for wasm, iOS and Android, so every package carries the transcoder
- [ ] Route Basis-encoded KTX2 payloads (BasisLZ or UASTC) through the transcoder
- [ ] Pick the target from the adapter: BC on desktop, ASTC on Apple, ETC2 on Android, `Rgba8Unorm` only when no compressed format exists
- [ ] Transcode each mip as it streams in
- [ ] Without the feature, Basis files fail with the decoder error from 0.13.0, which names `basis`, so the patch adds no `TextureError` variant
- [ ] A healthcheck that uploads one Basis fixture on every CI platform and compares the readback within a tolerance
- [ ] `docs/guides/basis-universal-workflow.md`, from the `basisu` CLI to loading the file in each language

### Backends and community

#### Backends

- [ ] A Canvas2D fallback for video filters, so something always renders
- [ ] `Renderer::capabilities()` reports compute, storage textures, immediates, compressed format families and similar features

#### Community shaders

- [ ] Registry submissions, checked before they go live
- [ ] Fork and remix: anyone can open a registry shader in the Shadertoy clone, edit it, and publish it under their own namespace
- [ ] Tags, search and trending lists
- [ ] Views, forks and stars per shader
- [ ] Moderation for infinite loops and abusive resource use

### Stability and performance

- [ ] A central device-lost handler that requests a new device, rebuilds the render context, reconfigures surfaces and targets, and resets caches and pools; docs on what survives; a simulated device-lost test where the backend allows it
- [ ] Staging-belt uploads for uniforms (`wgpu::util::StagingBelt`, or a per-frame flush in the buffer pool), honoring alignment rules
- [ ] Sampler and texture-view caches keyed by descriptor, with LRU caps
- [ ] Pipeline warming per shader, format and sample count, so a warm-up that knows its target builds pipelines the way `renderer.load` uploads data. An optional `wgpu::PipelineCache` owned by `Renderer` on Vulkan, the only backend that supports it
- [ ] Throttled counters and one-time warnings for frame-acquire retries on `Lost` and `Outdated`
- [ ] Texture worker: several workers, cancellation on drop, shader compiles and buffer uploads off the main thread

### Examples, tutorials and tests

#### Examples and tutorials

- [ ] Hello custom geometry (vertex input)
- [ ] Hello multiple screens
- [ ] Hello screen regions and viewports
- [ ] Build a simple image editor
- [ ] Hello Shader Organ: a post-process stack edited every frame (an FBM base with keyboard-toggled warp, chromatic, blur, pixelate, invert, vignette and feedback passes), exercising per-frame pass lists and `TextureTarget` reuse
- [ ] Step-by-step tutorials in `docs/api`, in the multi-block format
- [x] Big particle system (`particles`)
- [x] Simple multi-pass projected shadows (`multipass_shadows`)

#### Snapshot tests

- [ ] `Renderer::render_to_hash(&shader) -> Sha256`
- [ ] PNG diffs with a tolerance for differences between GPUs
- [ ] CI uploads the diff as an artifact
- [ ] Coverage: the circle shader, `compute_texture`, `particles` with a seeded RNG, storage texture pipelines, meshes with vertex and instance attributes, MSAA with resolve, immediates both native and through the uniform fallback
- [ ] A helper that snapshots a multi-pass render
- [ ] An `UPDATE_EXPECT=1` flag that rebaselines the snapshots, documented in CONTRIBUTING, and a macOS CI step
- [ ] Optional headless flags on the examples for snapshot tests

### Platforms

#### Mobile

- [ ] An iOS example app in `platforms/swift/examples/` that consumes the Swift package
- [ ] An Android example app in `platforms/kotlin/examples/` that consumes the AAR
- [ ] iOS and Android healthchecks that go past headless smoke tests: textures, immediates, frames
- [ ] Publish the Kotlin AAR to Maven Central
- [ ] Register the Swift package with the Swift Package Index

#### Python window integrations

- [ ] Qt
- [ ] WxWidgets
- [ ] Jupyter

#### All bindings

- [ ] A core `create_target_from_surface(surface, size)` helper shared by the web, Python, iOS and Android
- [ ] Expose the remaining `wgpu::RenderPass` options (load and store ops, the depth clear value) with sensible defaults. Stencil comes with 0.14
- [ ] JSON save and load: `Mesh::load_*` helpers, pass setups, shader values (uniforms, textures and override constants) and `Pipeline` records, with a JSON schema for defaults, and a JSON shader source. The JSON dependency sits behind a feature that is on by default, like every dependency feature

## 1.0.0: stable release

- [ ] Lock the public API and document the stability guarantees
- [ ] Assistant support: `llms.txt` and an [MCP](https://modelcontextprotocol.io/introduction) server, so coding assistants write correct FragmentColor code
- [ ] A polish pass over the website and docs
- [ ] Groundwork for translated docs

## Later: the 3D layer grows

Higher-level 3D features build on the 3D layer in the main crate and follow the layer rule. Each item is bound in all five languages, or ships as an example or an integration, and which one is decided per item when work starts. FragmentColor itself stays a library that an ECS drives, and ships no world or scheduler of its own. Skinned animation, glTF node animation, the unlit and toon presets and frustum culling moved to 0.13.x, and the cube-map `Camera`, shadow mapping and the glass preset to 0.14. The 2D UI and HUD and the gizmos are integrations.

- [ ] LOD, as an example first
- [ ] A GPU particle system (compute), as an example
- [ ] Optional Rapier physics integration
- [ ] FBX and other formats as the ecosystem asks. `ufbx` binds a C library, so its wasm build needs a C toolchain
- [ ] glTF material extras, `EXT_mesh_gpu_instancing`, which maps onto a mesh's instances, and `EXT_structural_metadata`, as the ecosystem asks
- [ ] Node animation from USD, after the glTF playback in the Characters track
- [ ] `KHR_animation_pointer`, which `gltf` 1.4.1 lacks
- [ ] Compute skinning as an example, once a compute pass can feed a mesh's vertices
- [ ] Runtime font baking with fontdue, about 3% of the web gzip, on request
- [ ] A `Sprite` handle or a 2D camera, on request
- [ ] A storage to uniform rewrite on WebGL2, only if data textures fall short
- [ ] Blending two libsm64 ticks on the GPU, from two vertex streams and a uniform `t`, if the harness's CPU blend falls short
- [ ] The walkable world playground demo loads its trees and props from glTF models, as a showcase of the loader

## Later: ready-made parts

Shaders compose from slugs today, as in `Shader::new("sdf/circle")`. Meshes, materials and models could come by name the same way, so a project, or an agent writing one, asks for a part instead of spelling out its geometry and shading. One call in place of a page of vertices saves tokens, and the program stays short enough to read. The M4 port shows which parts pay off first.

- [ ] Meshes by name, such as `Mesh::new("cube")`, with parameters for size and detail. `Mesh::new()` takes no argument today, so the call shape is decided when the work starts
- [ ] Procedural materials by name, such as `Material::new("metal")`, `"wood"` and `"water"`, built from registry WGSL and tuned through `set`
- [ ] Models by name, such as `Model::new("robot")`, assembled from parts that can be configured, and characters with names of their own, such as `Model::new("john_doe")`
- [ ] Prefabs that combine meshes, materials and shaders into one reusable piece, composable the way registry shaders are
- [ ] A study of what makes a 3D library easy for agents to drive: the calls and defaults that save the most tokens and still leave full control

## Backlog

Ideas without a target yet. They move up when someone starts on them.

- [ ] MSAA edge cases: a `resolve_target` per color attachment, beyond the current MSAA path
- [ ] Composition at the mixin level: merge several pass shaders into one stage, with a policy for binding conflicts and `use … remap(0, 1)` syntax
- [ ] Texture plugins that return bytes for `create_texture`
- [ ] Signed registry uploads to deter namespace squatting
- [ ] Halve the conversion traits with `Borrow<B>` across the `Into<UniformData>` and `Into<ColorTarget>` paths
- [ ] A Logo-like language for procedural shaders, delivered as a plugin
- [ ] Narrow slot formats such as `unorm8x4` colors through a doc-comment hint, since WGSL cannot mark a vertex input as normalized
- [ ] A loader option that fills instance slots from glTF `extras` keys
