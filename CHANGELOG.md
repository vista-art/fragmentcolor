# Changelog

## Unreleased

See the [Roadmap](https://github.com/vista-art/fragmentcolor/blob/main/ROADMAP.md) for planned features.

### `R16Unorm` (and the 16-bit norm family) now works on every adapter that advertises the feature

Diagnosed against RemixBrush's painting shader saga (`docs/PHASE-3.3.5a-runtime-saga.md` in the consumer repo): an `R16Unorm` `TextureMipChain` that round-tripped fine through `prepare → from_chain → device.create_texture` produced a silently-invalid texture on Apple Silicon, then exploded on first `create_view()` with an `InvalidResource` cascade that drowned the consumer's stderr 60 times per second. Same for `Rg16Unorm`, `Rgba16Unorm`, and the three `*Snorm` variants. Three layered fixes so the failure mode no longer reaches the user:

- **Adapter feature probe widened.** `request_device` now opportunistically requests `TEXTURE_FORMAT_16BIT_NORM` (and `FLOAT32_FILTERABLE`) alongside the texture-compression features it already negotiated. On every adapter that advertises the feature — every Apple Silicon device, every modern desktop GPU on Vulkan/Metal — `R16Unorm` + `TEXTURE_BINDING` now Just Works. The probe stays opt-in via `adapter.features().contains(...)` so adapters that don't advertise the feature still get a working device.
- **Fail-fast on adapters without the feature.** New `TextureError::UnsupportedFormatForUsage { format: wgpu::TextureFormat, missing_feature: wgpu::Features }` variant + `check_format(features, format, usage)` guard called at every `device.create_texture` site that takes a user-controlled format (`TextureObject::{new, from_input}`, the KTX2 loader). Consumers see a typed error at the API boundary — `"Texture format R16Unorm is not supported by the active device for the requested usage (missing wgpu feature TEXTURE_FORMAT_16BIT_NORM)"` — instead of the cascade-50-frames-later landmine.
- **wgpu validation scope around bind-group + view creation.** New `RenderContext::validate(label, op)` helper folds the prior `create_bind_group_checked` (which only wrapped `create_bind_group`) into a single generic that wraps any wgpu call whose validation failure would otherwise leak via `on_uncaptured_error` to stderr. `process_render_pass` and `process_compute_pass` now wrap both `tex.create_view()` and `device.create_bind_group()` with it, so consumers get one programmatic `RendererError::ValidationError { label, message }` instead of the 4-tier validation cascade `[Texture::create_view → InvalidResource] → [Device::create_bind_group → InvalidResource] → [set_bind_group → InvalidResource] → [Queue::submit → InvalidResource]` they used to flood logs with.
- **Regression test landed:** `renderer::tests::render_with_r16unorm_texture_smoke` exercises the full consumer path (R16Unorm prepared chain → bound via uniform → `renderer.render` → asserts no error). Plus pure-table unit tests for the lookup helpers (`format_feature_covers_16bit_norms_only`, `check_format_fails_fast_when_feature_absent`).

### Method naming pass — single canonical name per operation, no `_kind` / `_async` / `_object` / `_with_X` / `_checked` suffixes

Audit + cleanup landed across the public API and internal helpers, on top of the API-thinning section above. The rule: **1 verb or max 3 words; suffixes only when they disambiguate genuinely distinct inputs (`from_file` vs `from_bytes`)**. Internal helpers pay the same tax as the public surface. Consumer-visible (Rust-side) renames are listed below; platform binding suffixes (`_js` / `_mobile` / `_py` / `_android` / `_ios`) are forced by uniffi/wasm-bindgen/pyo3 needing distinct signatures and stay.

**Public Rust API (consumer-visible):**
- `Pass::add_mesh_to_shader(mesh, shader)` **removed**. The body was `shader.add_mesh(mesh)?` — a thin convenience that ignored `&self`. Callers use `shader.add_mesh(mesh)` directly. Deleted along with the per-platform wrappers (`add_mesh_to_shader_js/_mobile/_py`), the doc page (`docs/api/core/pass/add_mesh_to_shader.md`), and the four per-language example files.
- `PassObject::set_color_target_id(id)` → `set_color_target(id)`; `set_depth_target_id(id)` → `set_depth_target(id)`. The arg name carries the type.
- `App::on_event_kind(kind, f)` → `on_event(kind, f)`; `on_window_event_kind(id, kind, f)` → `on_window_event(id, kind, f)`; `on_device_event_kind(kind, f)` → `on_device_event(kind, f)`. The catch-all variants `on_event(f)` / `on_device_event(f)` (no `kind` arg) were removed entirely — kind-filtered registration is the only way; callers wanting every-event coverage register handlers per-kind. (The 1-arg / 2-arg overload via different names — `on_any_event` / `on_event` — was tried and rejected: "writing extra variants with `_any_` etc makes the file more confusing to read.")
- `create_external_texture_from_native(_r, _ptr)` (free fn) + `create_external_texture(_r, _video)` (free fn) → `ExternalTextureHandle::from_native(renderer, ptr)` + `ExternalTextureHandle::from_video(renderer, video)` (associated functions on the type). The implementation is still a stub — the API moves to where it belongs.
- `Target` trait gained `async fn get_image(&self) -> Vec<u8>` (was a sync method, then removed mid-refactor). Now async-only on the trait, mirroring `Texture::get_image()`. `TextureTarget::get_image_async` removed (the trait method covers it). `WindowTarget::get_image` is a stub returning `Vec::new()` for now — proper screen capture from a presentable surface needs `COPY_SRC` on the swapchain config, queued for v0.11.x; the API surface is uniform across every Target type today.

**`TextureObject` constructor family folded 5 → 1.** The five `pub(crate) from_*` constructors (`from_file`, `from_bytes`, `from_raw_bytes`, `from_image`, `from_chain` — the latter two renamed from `from_loaded_image` and `from_prepared_chain`) collapsed into a single async dispatcher `TextureObject::from_input(context, input)` matching on the `TextureData` variant. URL parts pre-fetch on the calling thread; everything else dispatches to the background worker. `Renderer::create_texture` shrank from ~165 lines (8-arm match + duplicated registration) to ~22 lines (handle `CloneOf` / `Empty` at the boundary, then delegate). The remaining three unique upload paths (`from_raw_bytes`, `from_image`, `from_chain`) stay as private helpers inside the impl. `from_file` and `from_bytes` deleted as trivial adapters (their bodies — `image::open` / `image::load_from_memory` — are inlined in the dispatcher's match arms).

**Sync/async pair unification (used the `blocking` submodule convention from `reqwest::blocking`).**
- `shader/input.rs`: `resolve_async` → `resolve` (async, top-level); the previous sync `resolve` moved to `blocking::resolve` (matching `reqwest::blocking`). Same for `resolve_part` and `fetch_url` helpers.
- `texture/read.rs`: `read_texture_object_async` → `read_pixels` (async). The previous sync `read_texture_object_sync` was the only consumer of the now-removed `Target::get_image` sync trait method; with `Target::get_image` now async, the sync read path is gone entirely. Internal `get_image_async` helper inside `texture/read.rs` lost its suffix to `get_image` (no companion to disambiguate).

**Internal helpers (renderer + texture + mesh):**
- `try_with_validation` → `validate` (renderer/mod.rs).
- `configure_surface_with_context` → `configure_surface` (renderer/mod.rs).
- `try_get_frame_with_retry` → `acquire_frame` (renderer/mod.rs).
- `create_vertex_buffer_layouts` → `vertex_buffer_layouts` (renderer/mod.rs).
- `create_bind_group_layouts` → `bind_group_layouts` (renderer/mod.rs).
- `available_compression_features` (then briefly `available_texture_format_features`) → `format_features` (renderer/platform/all.rs).
- `format_supports_cpu_mipmaps` → `supports_cpu_mipmaps` (texture/mod.rs).
- `build_mip_chain_bytes` → `build_mip_chain` (texture/mod.rs).
- `write_raw_bytes_levels` → `write_levels` (texture/mod.rs).
- `wrap_raw_bytes_as_dynamic_image` → `bytes_as_image` (texture/mod.rs).
- `infer_format_from_image` → `infer_format` (texture/mod.rs).
- `validate_format_for_binding` → `check_format` (texture/mod.rs); paired `required_feature_for_binding` → `format_feature`.
- `first_vertex_location_map` → `vertex_location_map` (mesh/mod.rs); `first_instance_location_map` → `instance_location_map`.
- `create_gpu_vertex_buffers` → `upload_vertex_buffers` (mesh/mod.rs).
- `Pass::from_shader_object` + `add_shader_object` (private internal duplicates of the public `Pass::from_shader` / `add_shader`) folded — the `PassObject` versions now take `Arc<ShaderObject>` directly and the public `Pass` wrappers do the `&Shader → Arc` extraction at the boundary.

### API thinning — single-method-per-operation across every binding, single transport across the API

A multi-slice refactor that:
1. Collapses the `_with_*` method families into single canonical methods that take a transport object with many `From<T>` impls (so Rust call sites stay tight);
2. Unifies the cross-language surface so JS / Python / Swift / Kotlin all see the same shapes; and
3. **Merges the three texture-input transports (`TextureSpec` / `StorageTextureInput` / `PrepareSpec`) into one shared `TextureInput`** so `create_texture`, `create_storage_texture`, and `TextureMipChain::prepare` all read from the same vocabulary.

Naming convention (post-rename): `TextureData` is the source enum (`Empty | Bytes | Path | Url | DynamicImage | Ktx2* | CloneOf | Prepared`); `TextureInput { data: TextureData, options: TextureOptions }` is the universal transport carried into every entry point; `TextureOptions` carries `size: Option<Size>`, `format`, `sampler`, `mipmaps`, and `usage: Option<u32>` (raw bit mask, with a `with_usage(wgpu::TextureUsages)` builder for typed Rust call sites).

- **`Renderer::create_texture(input)` is the single texture-creation entry.** Drops `create_texture_with_size`, `create_texture_with_format`, `create_texture_with`, and `create_texture_prepared` — every shape now goes through `create_texture(impl Into<TextureInput>)`. JS / Python collapse to one method with an optional `options` arg; mobile takes a uniffi-marshallable `TextureInputMobile` enum + optional `TextureOptions` (now a `uniffi::Record`); Swift / Kotlin extension files supply natural overloads.
- **`Renderer::create_storage_texture(input)` is the single storage-texture entry.** Drops `create_storage_texture_with_data` and the separate `StorageTextureInput` transport — same `TextureInput` as `create_texture`, with the discriminator `data: TextureData::Empty` (no initial data, just allocate) vs. `data: TextureData::Bytes(...)` (pre-seed). The `From<(size, format)>` impl produces the empty form; `From<(size, format, bytes)>` produces the seeded form. `options.usage` overrides the default storage-usage mask.
- **`TextureMipChain::prepare(input)` is the single CPU-prep entry.** Drops the separate `PrepareSpec` transport — `prepare` takes the same `TextureInput` as the renderer methods. Tuple `From` impls cover the common shapes (`(bytes, format)` for encoded, `(bytes, format, size)` for raw); `prepare` validates `data` is a sync-friendly variant (`Bytes`, `DynamicImage`, `Path`) and surfaces a typed `InvalidInput` error pointing at the right entry point for the variants it can't handle (`Url` → fetch first, `Ktx2*` → already pre-baked, `Prepared` → already a chain, `Empty` → nothing to prepare).
- **`Renderer::render(renderable, target)` is the single render entry on every platform.** The mobile uniffi binding used to ship `renderShader` + `renderShaderToTexture` because uniffi can't marshal `&impl Renderable` / `&impl Target`; now `RenderableHandle` (`Shader | Pass | Mesh | Passes`) and `TargetHandle` (`Window | Texture`) `uniffi::Enum`s carry the dispatch, and Swift / Kotlin extension files supply natural overloads (`renderer.render(shader, target)` / `(pass, target)` / `(mesh, target)` / `(passes, target)`) that wrap the concrete handle into the matching variant invisibly. End users never see the mobile-only mirror types. `Pass` and `Mesh` now derive `uniffi::Object` so they can ride inside the handle enums.
- **Cross-language brand detection for `TextureMipChain` handles in JS.** Reused the codebase's existing `__fc_kind` + `__wbg_ptr` anchor pattern (see `impl_js_bridge!`) so `await renderer.createTexture(chain)` finds the chain in the dispatched `TextureData` without `dyn_ref` (which doesn't work for wasm-bindgen-exposed types).
- **Net surface delta:** ~9 Rust methods → 4. ~24 FFI shims → ~9. Three transport types → one. Every `create_texture` / `create_storage_texture` / `prepare` / `render` call site now reads the same way on Rust, JS, Python, Swift, and Kotlin, and the same `TextureInput` flows through all three texture paths.
- **Trade-off accepted for the unification:** "size required for storage" and "data must be sync-friendly for prepare" are runtime validations rather than compile-time guarantees. Same convention as the existing KTX2 paths silently ignoring `options.format` / `options.mipmaps`.
- **Follow-up noted (not in this change):** structurally splitting `src/renderer/platform/mobile/` into per-language `ios.rs` + `android.rs` so each language's idioms get their own translation layer. The render unification above doesn't require it (Swift / Kotlin extension files already live in `platforms/swift/` and `platforms/kotlin/`), and bundling that split into this change risks over-scoping.

### Texture creation off the main thread (transparent on native, opt-in escape hatch on every language)

- **`Renderer::create_texture` no longer blocks the calling thread on CPU work.**
  Decoding (`image::load_from_memory` / `image::open`), the
  `image::imageops::resize` Triangle-filter mipmap chain, and the per-level
  `wgpu::Queue::write_texture` calls now run on a single named worker
  (`fragmentcolor-bg`) on every native target. The public API is unchanged —
  `let tex = renderer.create_texture(bytes).await?` still returns a `Texture`
  whose GPU writes are submitted by the time the future resolves — but the
  caller's thread (typically the GPU / event-loop thread) is no longer pinned
  for the ~30–50 ms a 512×512 RGBA tile costs. Affects the `Bytes`, `Path`,
  `Url` (after the async fetch), and `DynamicImage` arms; KTX2 inputs are
  cheap to decode and stay inline for now. The worker is process-wide and
  lazy — first `create_texture` call spawns it, no thread pool to configure,
  no runtime to pick.
- **Wasm keeps today's behavior.** `wgpu::Device` and `wgpu::Queue` are
  `!Send` on `wasm32` (they hold JS objects bound to the page's main thread),
  so the `cfg(wasm)` path runs the prep inline. No regression vs. previous
  releases; web users who need real parallelism can move decode + prep into a
  Web Worker themselves.
- **New `TextureMipChain` type, available on every language binding.** Builds
  a CPU mipmap chain off the renderer thread so callers driving their own
  decode pipeline (RemixBrush's tile cache, anyone using rayon / Swift
  `Task` / Kotlin `Dispatchers.Default` / Python `ThreadPoolExecutor` / a
  Web Worker) can fold the mipmap pass into the same hop. Two constructors:
  - `TextureMipChain::prepare(bytes, format)` — encoded image bytes
    (PNG/JPEG/etc.); decodes internally with the `image` crate.
  - `TextureMipChain::prepare_raw(bytes, size, format)` — raw pixel bytes
    already laid out for the format. The path RemixBrush uses after JPEG
    decode.
  Supported formats match `format_supports_cpu_mipmaps` (Rgba8/Bgra8 Unorm +
  Srgb, R8, Rg8, R16, Rg16, Rgba16); other formats return a clear error.
  The chain is consumed via `Renderer::create_texture_prepared(chain)` (the
  cross-language entry point) or `Renderer::create_texture(TextureInput::Prepared(chain))`
  (Rust ergonomics, same internals). `TextureMipChain` derives `Clone` via an
  internal `Arc<Vec<Vec<u8>>>` so handing the same chain to multiple textures
  doesn't duplicate the byte buffers.
- **Cross-language exposure**: `TextureMipChain` is bound via
  `#[wasm_bindgen]` (Web), `#[pyclass]` + `#[staticmethod]` (Python), and
  `#[uniffi::constructor]` (Swift / Kotlin via uniffi). Constructors return
  the type; accessors (`format()` / `baseSize()` / `levelCount()` / `level(i)`)
  let callers inspect or persist a chain. To make the uniffi side work,
  `TextureFormat` now derives `uniffi::Enum` and `Size` derives
  `uniffi::Record`; both already had Web/Python bindings, so existing call
  sites are unaffected.
- **Typed error surface for the prepare path.** `TextureError` gained two
  variants so callers (RemixBrush's tile-cache logger, anyone funneling
  prepare failures into a single user-facing message) can tell at a glance
  what went wrong on a corrupt tile vs. a misconfigured pipeline:
  - `TextureError::MalformedImageError(image::ImageError)` — the input bytes
    couldn't be decoded as an image. (Variant existed; the doc comment is
    new and now explicitly contrasts it with the other two.)
  - `TextureError::UnsupportedMipmapFormat { format: TextureFormat }` — the
    bytes were fine but the requested target format isn't supported by the
    CPU mipmap dispatcher. The variant carries the public `TextureFormat`
    so callers can match without reverse-engineering a string.
  - `TextureError::InvalidInput(String)` — the bytes parsed but didn't
    match the declared shape (zero size, byte count too small for
    `bpp * width * height`, etc.). Distinct from `MalformedImageError`.
- **`prepare_raw` accepts `impl Into<Size>`** on the canonical Rust
  signature, matching the `create_texture_with_size` pattern. Callers can
  pass `(w, h)`, `[w, h]`, or a bare `Size`; cross-language bindings still
  take a concrete `Size` (uniffi / wasm-bindgen / pyo3 don't marshal
  generics).
- **No new dependencies.** The worker uses `std::thread` + `std::sync::mpsc`
  for the job queue and `futures::channel::oneshot` (already a dep) for the
  per-call reply.
- **What's intentionally out of scope:** multi-worker pool (one worker fully
  unblocks the main thread, which is the primary ask; we'll scale to N when
  batch throughput becomes the limiter), drop-cancellation (orphaned futures
  let wgpu drop the texture on its own — fine for the typical caller),
  shader-compile / buffer-upload offload (same pattern would apply, no
  reported bottleneck yet), `TextureInput` marshalling across FFI (cross-language
  callers use the dedicated `create_texture_prepared` entry point instead).

### KTX2 container support (BC / ETC2 / ASTC + uncompressed)

- **`TextureInput` gained three KTX2 variants** — `Ktx2Bytes(Vec<u8>)`,
  `Ktx2Path(PathBuf)`, and `Ktx2Url(String)` — so consumers with an asset
  pipeline that emits `.ktx2` files (BC7 on desktop, ASTC on mobile / WebGPU,
  ETC2 on Android, uncompressed RGBA8/RGBA16F as a fallback) can load them
  through the same `Renderer::create_texture(_with)` entry points as JPEG/PNG
  sources. Pure-Rust parsing via the `ktx2` crate; no C++ build pollution.
- **The KTX2 path trusts the file's declared format and pre-baked mip chain.**
  We don't second-guess the encoder's sRGB choice or run our CPU
  `imageops::resize` chain — both `options.format` and `options.mipmaps` are
  intentionally ignored for KTX2 inputs. Encoders pick the format and chain
  on purpose; doing it twice would only round-trip through a worse
  approximation.
- **Compression GPU features are requested opportunistically at device
  creation.** The renderer now asks for whatever subset of
  `TEXTURE_COMPRESSION_BC` / `_ETC2` / `_ASTC` (and the SLICED_3D / HDR
  variants) the active adapter advertises. Adapters without a given feature
  still get a working device; KTX2 loads of formats the GPU can't sample fail
  at upload with a clear error rather than crashing inside wgpu validation.
- **Format coverage** (mapped from Vulkan `VkFormat` to `wgpu::TextureFormat`):
  RGBA8 UNORM/SRGB, BGRA8 UNORM/SRGB, R8/Rg8/R16/Rg16/Rgba16 UNORM, RGBA16F,
  BC1–BC7 (UNORM and SRGB variants), ETC2 RGB/RGBA/RGB-A1 (UNORM and SRGB),
  ASTC 4×4 and 8×8 (UNORM and SRGB). Other VkFormats fail loudly so consumers
  see them; extending the table is a one-line change per format.
- **Out of scope (deferred to follow-up PRs only when needed):** Basis
  Universal transcoding (`VK_FORMAT_UNDEFINED` payloads), supercompression
  schemes (zstd / zlib / BasisLZ), cube maps, array textures, 3D textures,
  and progressive intra-file mip streaming. The use case driving this PR
  ships per-tile complete `.ktx2` files; tile-level streaming lives at the
  consumer's viewer layer, not inside this loader.

### Wider source-image format support (R8 / Rg8 / R16 / Rg16 / Rgba16)

- **`Renderer::create_texture` now decodes images into the right pixel buffer for
  the target format**, instead of going through `to_rgba8` for everything. A
  16-bit grayscale PNG loaded with `format: TextureFormat::R16Unorm` is now
  re-decoded with `to_luma16` so the upper 8 bits aren't truncated — useful for
  height maps, mask buffers, and other high-precision single-channel data. The
  same dispatch handles `R8Unorm` (via `to_luma8`), `Rg8Unorm` (`to_luma_alpha8`),
  `Rg16Unorm` (`to_luma_alpha16`), and `Rgba16Unorm` (`to_rgba16`). Mipmap
  generation runs over the typed `ImageBuffer` for each of those formats too,
  preserving precision at every level.
- **`TextureFormat` gained `R16Unorm` and `Rg16Unorm` variants** so callers can
  request 16-bit single- and dual-channel textures from any binding (Rust, JS,
  Python, Swift, Kotlin) without dropping to raw bytes. The numeric ordering of
  the JS bridge enum shifted to insert the new variants alongside the other
  8-bit formats — JS callers passing format integers directly should re-read
  the values from the regenerated bindings.
- **`from_raw_bytes` mipmap support generalized** to the same set of formats.
  16-bit byte slices are decoded to `Vec<u16>` via `from_le_bytes` before
  resampling — alignment-safe, matches WebGPU's little-endian element order.
- Pre-existing bug fixed as a side effect: previously, a 16-bit PNG fed through
  `create_texture(path)` would be created with format `R16Unorm` (per the
  `image::ColorType` inference) but written with `to_rgba8` bytes (4 bpp into a
  2-bpp texture), producing garbled rows. The new dispatch makes the inferred
  format and the byte layout actually agree.

### Source-image mipmaps + trilinear filtering

- **`Renderer::create_texture` and friends now generate a full mipmap chain at upload** for
  source images (file path, encoded bytes, URL, `DynamicImage`). Combined with the existing
  default linear sampler — which also picks `mipmap_filter: Linear` when `smooth: true` —
  textured surfaces now get proper trilinear filtering at any zoom or rotation. Fixes the
  classic "moving moiré" artifact when zooming out on a textured quad whose source image
  has high-frequency detail (canvas weave in painted JPEGs being the canonical case).
  Downsampling happens CPU-side via `image::imageops::resize` with the Triangle filter;
  resampling runs directly on the source bytes (sRGB-encoded for color content) — visually
  close to a gamma-correct pass and dramatically better than no mipmaps. No GPU work is
  added at render time.
- **`TextureOptions` gained `mipmaps: bool` (default `true`).** Set to `false` to skip the
  CPU work for textures that won't be sampled at distance (single-pixel sentinels, render
  targets you'll only sample 1:1, etc.). Existing call sites that build `TextureOptions`
  with `..Default::default()` get mipmaps automatically.
- **`TextureOptions.format` is now honored on every input arm** of `Renderer::create_texture_with`,
  not just the raw-bytes-with-size path. The sentinel default `TextureFormat::Rgba` still
  means "infer from input" (preserving the prior behavior — a JPEG/PNG keeps loading as
  `Rgba8UnormSrgb`); any explicit variant becomes a real override. This unblocks loading
  RGBA8 PNGs as `Rgba8Unorm` (linear bytes, no sRGB conversion at sample time) for
  normal-map / non-color data without the `from_raw_bytes` round-trip.
- **`TextureObject` constructors generalized in place.** `from_file`, `from_bytes`,
  `from_raw_bytes`, and `from_loaded_image` each gained `(format_override, generate_mipmaps)`
  trailing parameters. They're `pub(crate)` so this is internal only — the dead pre-existing
  no-arg wrappers got pruned rather than kept as forwarders.
- Mipmap generation only runs for color formats that share the source's RGBA8 byte layout
  (Rgba8 family + Bgra8). Other formats stay single-level even when `mipmaps: true`.

### Shader composition

- **`Shader::new` now accepts arrays.** The signature is `Shader::new(impl Into<ShaderInput>)`,
  with `From` impls that classify a single string into one of: raw WGSL/GLSL **source**, registry
  **slug** (`"sdf2d/circle"`), `https://` **URL**, or local **path**. Pass an array of any
  combination — parts are resolved (fetched / read / looked up), deduplicated by source hash,
  and concatenated in order before naga validation. No wrapping or auto-injection: invalid
  input fails loudly.
  - Equivalent forms: `Shader::new(src)`, `Shader::new("sdf2d/circle")`,
    `Shader::new(["sdf2d/circle", "noise/simplex2", main_src])`.
  - Existing call sites that pass `&str`, `String`, or `&String` keep working unchanged.
- **`Shader::set_registry(base_url)`** overrides the slug base URL (default
  `https://fragmentcolor.org/shaders/`). Process-wide; tests use a thread-local override stack.
- **GLSL** is supported only as a single part; mixing GLSL with WGSL or with other parts is rejected.
- **Behaviour change:** a string of the shape `^[a-z][a-z0-9_]*/[a-z0-9_]+$` (single line, ≤128
  chars) is now classified as a **slug** rather than parsed as raw WGSL. Existing
  `Shader::new("sdf2d/circle")` callers will fetch from the registry instead of erroring.
- **JS/WASM:** `new Shader(input)` accepts `string | string[]`. URL/slug parts must be resolved
  via `await Shader.fetch(input)` (which now also accepts `string | string[]`). New
  `Shader.setRegistry(baseUrl)` static method.
- **Python:** `Shader(input)` accepts `Union[str, list[str]]`. New static `Shader.set_registry(base_url)`.
- **Swift / Kotlin (uniffi):** `Shader.new(source)` and new `Shader.compose(parts)` constructors,
  plus a free `set_shader_registry(base_url)` function. Extension shims provide a single
  overloaded `Shader(_:)`.

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
  still a stub returning `RendererError::Error("not implemented yet")` on every platform —
  the API surface is in place so callers can write portable code paths now and the
  per-platform plumbing fills in over 0.12.6 (see ROADMAP). `ExternalTextureHandle` is no
  longer `#[cfg(wasm)]`-gated — it derives `uniffi::Object` on mobile and stays
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
  already accepts `&Pass`, `&Vec<Pass>`, `&[&Pass]`, `&[Pass]`, and `&Vec<&Pass>` — every
  Frame use-case transliterates directly. Because every public symbol multiplies across
  5 language bindings (Rust, JS, Python, Swift, Kotlin), the API surface reduction is
  worth the transliteration cost — and 0.11.0 is not yet published, so no consumers exist.
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
- `Renderer::create_texture_target_mobile(width, height)` — uniffi-friendly concrete-typed
  variant of `create_texture_target`; Swift / Kotlin see it as `createTextureTarget`.
- `Renderer::render_shader_mobile` / `render_shader_texture_mobile` — uniffi variants of
  `render` (one per target type, since uniffi can't marshal `impl Trait`). Swift / Kotlin
  extensions recombine them into a single overloaded `render(shader, target)`.
- `Shader::new_mobile(source: String)` — uniffi constructor; Swift sees it as `Shader(source:)`
  via uniffi's `convenience init`, Kotlin sees it as `Shader.new(source)`.
- `Texture::set_sampler_options_mobile(opts: SamplerOptions)` — uniffi shim; Swift sees it as
  `setSamplerOptions(opts:)`, Kotlin as `setSamplerOptions(opts)`. Closes the parity gap
  with the existing Web (`setSamplerOptions`) and Python (`set_sampler_options`) wrappers.
- `Renderer::create_external_texture_mobile(source_ptr: u64)` — uniffi shim that takes a
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
but those generated files were only displayed on the website — nothing compiled or
ran them. JS and Python examples were already aggregated into runnable healthchecks
(`platforms/web/healthcheck/generated_examples.mjs` + `platforms/python/examples/main.py`);
Swift and Kotlin had no equivalent, so transpiler regressions and missing uniffi exports
went unnoticed until they reached the website.

`scripts/website.rs::write_healthcheck_aggregators` now also emits two compile-only
aggregators that embed every generated example body inside a private wrapper function:

- `platforms/swift/healthcheck/GeneratedExamples.swift` — picked up by the existing
  SPM executable target. `./healthcheck ios` was split into two sub-tests
  (`platforms.swift.bindings` + `platforms.swift.examples`); the second runs
  `xcodebuild -scheme fragmentcolor-healthcheck -destination 'generic/platform=iOS Simulator' build`
  and fails if any embedded body fails to type-check.
- `platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/GeneratedExamples.kt`
  — placed under `androidTest` so the existing
  `gradle fragmentcolor:connectedAndroidTest` invocation in `./healthcheck android`
  compiles the wrappers as part of the test source set. No CI YAML change needed.

Compile-only on day one — runtime execution requires a live GPU surface and is
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
  texture pre-seeded from a CPU blob in one call — skips the "author a trivial seed WGSL shader"
  workaround for initial conditions. Expects tightly-packed bytes (no per-row padding) so small
  textures work without manually padding rows to 256.
- `Renderer::wait()` blocks until every queued submission on the device has finished.
  Restores deterministic ordering around `render()` → readback sequences — previously a compute
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
  read-only storage buffers. Previously compute shaders sampling a non-storage texture — or using
  a sampler to filter one — failed pipeline creation with "Shader global ResourceBinding ... is not
  available in the layout / Visibility flags don't include the shader stage", forcing workarounds
  like declaring sources as `texture_storage_2d<..., read>`.
- `texture_storage_2d<..., read_write>` is no longer silently downgraded to `ReadOnly`. Naga
  represents `read_write` as `StorageAccess::LOAD | STORE`, which previously fell through the match
  arm and produced a read-only bind group layout — any `textureStore` then failed validation. The
  mapping now emits `StorageTextureAccess::ReadWrite` for the combined case, allowing ping-pong
  pairs to collapse to a single texture where the format supports it.
- `Texture::get_image_async` no longer deadlocks on native. The async path now drives
  `device.poll(Wait)` before awaiting the map callback — without it the oneshot future waits
  forever because nothing else advances the wgpu event loop on non-web targets.

### Platform workarounds

- Apple Silicon: on `macos` / `ios` targets the renderer now submits the current command buffer
  between two sequential compute passes, then records the next compute pass on a fresh encoder.
  The submission boundary reliably flushes Metal's tile-based storage-texture writes so a
  subsequent `texture_2d<f32>` / `textureLoad` in the next compute pass observes the results —
  previously this pattern silently returned zeros. Compile-time routed via a new `apple` cfg
  alias (`target_os = "macos" | "ios"`); users do not need to opt in, and non-Apple targets are
  unaffected. The previous workaround (declaring the source as `texture_storage_2d<..., read>`)
  is no longer required.

### Transpiler — Rust-idiom scrubbing for Swift / Kotlin / JS / Python

The `docs/api/**.md` Rust examples are transpiled to four targets. The previous
output inherited Rust syntax in several cases — Swift and Kotlin saw it loudest
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
- **Standalone `let var = r#"..."#`** — multi-line bare raw-string assignments
  (not just `Type::new(r#"..."#)`) are now gobbled and re-emitted as a single
  triple-quoted (Swift / Kotlin / Python) or backtick (JS) string. Used by the
  shader-composition example.
- **Single-quoted JS string literals → double-quoted** in Swift and Kotlin
  output. Outside-strings detection so apostrophes inside WGSL comments survive.
- **Kotlin `[a, b, c]` collection literals → `arrayOf(a, b, c)`** because Kotlin
  only accepts `[...]` syntax in annotation arguments. Indexer patterns
  (`arr[0]`) are not rewritten.
- **Source-level overrides** for examples that have no idiomatic Swift / Kotlin
  equivalent — added per-language `hidden/<file>_<lang>.md` for
  `Renderer::create_external_texture_from_html_video` (wasm-Rust-only) and
  `Texture::set_sampler_options` (uniffi gap). Both render as a stub comment
  on the website.

After this pass the Swift aggregator parses cleanly under `swiftc -parse` (down
from 50 parse errors). Type-check / compile errors remain — the per-doc Rust
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
  - **`headless_window`-derived JS DOM leak** — Rust source uses `headless_window([w, h])`
    + `renderer.create_target(window)`, which today maps to `document.createElement('canvas')`
    (JS-specific). Swift / Kotlin output now parses (single quotes swapped) but `document`
    doesn't exist on those platforms. Either the source examples need to use
    `create_texture_target([w, h])` (portable across all four languages and uniffi-exported)
    or per-language `hidden/_swift.md` / `_kotlin.md` overrides need a CAMetalLayer /
    SurfaceView snippet.
  - **`Shader.default()` / `Shader.fromMesh(mesh)`** are not uniffi-exported — Swift / Kotlin
    examples reference methods that don't exist. Either expose them via uniffi or rewrite
    the source examples to use `Shader::new(source)`.
  - **Rust `[a, b]` `impl Into<Size>` vs uniffi flattened signatures** — many examples pass
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
  a texture and a subsequent render pass in the same command buffer samples it — the render pass's
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
      `connectedAndroidTest` flow — compile-only, mirroring the JS / Python coverage that
      already existed for the same per-doc transpiled output
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
- [x] `wgpu` / `naga` upgrade from 27.0.1 to 29.0.1 (full adapter — see _Dependency Updates_)

### Carried over to 0.12.0

- [ ] Example iOS app under `platforms/swift/examples/` (xcodeproj consuming the SPM package)
- [ ] Example Android app under `platforms/kotlin/examples/` (gradle project consuming the AAR)
- [ ] Expand mobile healthchecks beyond the headless smoke test (textures, immediates, frames, …)
- [ ] Drain the Swift / Kotlin doc-example punch list surfaced by the new aggregators (see
      _Known issues_). Round 1 (parse errors) is done — Swift now parses 0 errors. Round 2
      (type / compile errors) needs per-language emission with uniffi-signature awareness:
      detect `createTextureTarget([w, h])` and rewrite to positional args, replace
      `headless_window` with `create_texture_target` at the source level (or per-language
      override), and either expose `Shader::default` / `Shader::from_mesh` via uniffi or
      rewrite the offending source examples. Then promote both aggregators from compile-only
      to runtime execution.
- [ ] Platform-side cleanup of stale generated examples under `platforms/{web,python,swift,kotlin}/examples/`
      when their source `docs/api/*.md` is deleted (today the website MDX is pruned but the
      per-platform sources are not)
- [ ] Implement `Renderer::create_external_texture` for real on every platform — the API
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
