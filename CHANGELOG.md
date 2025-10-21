# Changelog

## Unreleased

See the [Roadmap](https://github.com/vista-art/fragmentcolor/blob/main/ROADMAP.md) for planned features.

## 0.10.10 Web glue guard, ArrayBuffer handling fixes, and website hero cleanup

### Build system

- Web (WASM): add a post-bindgen patch step in `build_web` that hardens the generated glue.
  - Guard the `Uint8Array(ArrayBuffer)` constructor used by wasm-bindgen shims against detached ArrayBuffer.
  - On failure, fall back to `new Uint8Array(wasm.memory.buffer)` (live memory) to avoid crashes in long prod runs.

### Web (WASM)

- TextureInput (JS bridge): make ArrayBuffer handling robust on Web — treat `byte_length() == 0` as detached/empty and return an empty byte vector instead of throwing.
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
