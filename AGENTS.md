# AGENTS.md

This file is the top-level guide for any AI coding agent (Claude Code, WARP, Cursor, Copilot CLI, …) working in this repository. Per-module rules live in sibling `AGENTS.md` files (see the "Module-level invariants" section below).

Project summary
- Crate: fragmentcolor (library) with examples in examples/rust (crate fce).
- Languages: Rust core with generated bindings/examples for Web (WASM/JS), Python, Swift (iOS), and Kotlin (Android).
- Toolchain: rust-toolchain set to stable with clippy/rustfmt and common cross targets (desktop, wasm, iOS aarch64 + sim, all 4 Android ABIs).

Common commands
- Repository scripts live in `tools/`; `./fc` lists them and `./fc <tool> [args]` runs one.
- Rust core
  - Build: `cargo build`
  - Test (workspace): `./fc test` (runs clippy, all Rust tests, and crate doctests)
  - Test (single):
    - Unit/integration by name: `cargo test -p fragmentcolor <name> -- --exact --nocapture`
    - Specific integration file: `cargo test -p fragmentcolor --test e2e <name>`
  - Examples: `./fc example` opens a picker and `./fc example triangle` runs one by name; `--release` builds it optimized, and arguments after the name go to the example
- Lint & format
  - Fast check: `cargo clippy --all-targets --all-features -- -D warnings`
  - Preferred flow: `./fc clippy` (formats first); autofix: `./fc clippy fix`
- Cross‑platform healthchecks (generated samples)
  - All: `./fc healthcheck`
  - Web only: `./fc healthcheck web` (uses Playwright; respects `PORT`, `FC_HEALTHCHECK_VERBOSE=1`)
  - Python only: `./fc healthcheck py`
  - iOS only: `./fc healthcheck ios` (runs Swift examples against the generated xcframework on an iOS simulator — requires Xcode)
  - Android only: `./fc healthcheck android` (runs Kotlin examples against the generated AAR on an Android emulator — requires cargo-ndk + Android SDK + emulator image)
- Web (WASM)
  - Build pkg: `./fc build_web` (debug: `./fc build_web --debug`)
  - Dev servers: `./fc run_web [gallery|repl|visual]` (respects `PORT`, `REPL_PORT`)
  - Sync JS deps: `./fc sync_js`
- Python
  - Local run helper: `./fc run_py [main|multiobject|headless]` (builds wheel, manages venv)
  - Manual build: `pipx install maturin && ./fc build_py`
- iOS (Swift)
  - Build xcframework + regenerate Swift bindings: `./fc build_ios`
  - Produces `build/ios/FragmentColorFFI.xcframework` and Swift sources under `platforms/swift/Sources/FragmentColor/generated/`
  - Commit the regenerated `generated/FragmentColor.swift` whenever the uniffi surface changes. SPM consumers compile it from the release tag, and the iOS healthcheck fails when it differs from `build_ios` output.
  - `platforms/swift/examples/` holds the Swift examples transpiled from docs/api. There is no runnable Swift app yet.
- Android (Kotlin)
  - Build `.so` per ABI + regenerate Kotlin bindings: `./fc build_android [TARGET]`
  - Requires: `cargo install cargo-ndk` and `ANDROID_NDK_HOME` pointing at a valid NDK
  - Produces `jniLibs/<abi>/libfragmentcolor.so` plus Kotlin sources under `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/generated/`
  - `platforms/kotlin/examples/` holds the Kotlin examples transpiled from docs/api. There is no runnable Android app yet.
- Docs site (Astro/Starlight)
  - Dev: `./fc run_docs` (port via `--port` or `DOCS_PORT`)
  - Preview (built): `./fc run_docs preview`

Big‑picture architecture (how things fit together)
- Public API surface (src/lib.rs)
  - Core types are re‑exported at the crate root: Renderer, Shader, Pass, Texture, Target (WindowTarget/TextureTarget), Mesh, Vertex, Size, Color, ScreenRegion, plus stable kind branding via fc_kind.
  - Platform shims live under each module’s platform/ submodule (python, web, winit, etc.).
- Renderer and context
  - Renderer lazily creates a RenderContext (wgpu::Device/Queue). It caches render/compute pipelines keyed by descriptive structs; manages uniform/storage/readback buffer pools and a small texture pool for transient MSAA.
  - In Rust, create_target prefers a surface; on failure it logs and falls back to an offscreen TextureTarget so headless/CI runs still work. The bindings don't fall back: their create_target returns the error. MSAA for surface targets uses transient textures resolved into the swapchain view.
  - Uniforms/textures are reflected from Shader storage. Textures are registered in a Renderer‑owned registry and referenced by an integer TextureId. A Material map set from a source keeps that source on its shader, and each Renderer that loads it holds a copy of its own under the map's one TextureId.
  - Push constants: native push constants are used when available; otherwise they are lowered to a fallback uniform buffer per root on platforms without push‑constant support (e.g., Web).
- Shader, Mesh, Pass
  - Shader owns attached meshes and validates every vertex and instance at attach time. Typed Rust values (`f32`, `u32`) must match the input's format exactly; plain numbers take the type the shader declares, and the first shader that types an attribute fixes it for the mesh. Fullscreen shaders (no @location vertex inputs) reject mesh attachments.
  - Pass is a thin orchestrator for shaders and render‑time knobs (viewport, clear color, compute dispatch, optional per‑pass targets). A pass is compute if all attached shaders are compute. Passes build DAGs via `Pass::require()`; any iterable of Pass is Renderable, so ordered sequences just pass a slice to `Renderer::render`.
- Documentation‑driven pipeline (THIS IS LOAD-BEARING)
  - Canonical docs live in docs/api and are pulled into Rust via #[lsp_doc(...)] so IDE hovers match the website.
  - build.rs enforces a “no‑panic in library code” policy, scans the public API to generate generated/api_map.rs and language examples, writes healthcheck aggregators, exports website pages, and generates the Python type stub (`platforms/python/fragmentcolor/fragmentcolor.pyi`, see `scripts/python_stubs.rs`).
  - **That pipeline runs only in a repository checkout**: a `.git` (a directory, or a file in a worktree) next to `Cargo.toml`, with `DOCS_RS` unset. A packaged build (crates.io, docs.rs, the Python sdist, a vendored copy) may sit on a read-only filesystem, so it writes only to `OUT_DIR`. Anything the crate compiles from a build output, such as the embedded shader lookup, goes to `OUT_DIR` and is included from there, never from the source tree. A new file the crate reads at compile time (an `#[lsp_doc]` page outside `docs/api`, an `include_str!` target) must also be added to `include` in `Cargo.toml`, or the published crate fails to build.
  - **Generators read API facts from the source, not from hand-kept tables.** `scripts/reflect.rs` walks the module tree from `src/lib.rs` once, decides each item's `cfg` per build (Python wheel, wasm, iOS, Android), and expands the crate's own `macro_rules!` calls. `ApiCatalog` carries the result to every generator. The Python stub takes every hint from it, and a parameter takes NumPy values as rule 6 says: an int is `SupportsIndex`, a float `SupportsFloat`, and a list of numbers also an `npt.NDArray`. The transpilers take from it which calls throw or are async, which arguments are floats, unsigned ints, lists, or flat lists of numbers, the flat enum spellings, and what each binding exports. They also take how a tuple argument spreads over a binding's parameters (the uniffi parameters, or the pyo3 `signature` for Python keywords), which records an array literal fills in field order, and which mobile-only type stands in for a crate type, such as `TextureInputMobile` for `TextureInput`. They read the Swift and Kotlin overloads from the hand-written `*Extensions` files through `scripts/source_text.rs`. New generator code must not list API names, types, or signatures by hand; when it needs a fact the model lacks, extend `scripts/reflect.rs`. The per-call rewrites still in `convert.rs` and `kotlin.rs` predate this rule and are not a pattern to copy. They stand in for a receiver or local type a line rewriter can't know (Python's `.size` property, Kotlin's byte-array locals). Platform setup, such as a canvas or a file read, lives in each language's hidden block in the docs/api example, never in a transpiler (see docs/api/AGENTS.md). When a Kotlin call needs a rewrite that the Swift one doesn't, add the overload the Kotlin extension files lack instead.
  - Tests include crate doctests and harness checks that validate the generated JS/Python examples (e.g., string formatting and WGSL integrity) to keep docs/examples and code in lockstep.
  - **Invariant:** any public item (object or method) visible to at least one binding must have a docs/api page with a `## Example` section, an `#[lsp_doc(...)]` attribute on the Rust item, and a working transpiled example for every language that exposes it.
- Cross-language guarantees (the whole point)
  - **Documentation is always up to date** and derived from docs/api — never hand-edit the website, healthcheck examples, or platform-specific docstrings. The website is generated from docs/api; the healthcheck examples are generated from docs/api; Rust hover docs are pulled from docs/api; the Python stub's hints come from the Rust sources through `scripts/reflect.rs` (a binding's own parameter types, or the canonical method it documents, followed through the crate's conversions) and its docstrings come from docs/api.
  - **The API looks the same in every language, in every release.** "Looks the same" has a precise meaning; see [What "looks the same" means](#what-looks-the-same-means) below.
  - **100% API parity is enforced at build time** across Rust, Python, JavaScript, Swift, and Kotlin. `build.rs` fails if a public object exists in the Rust API without a doc page, if a method lacks `## Example`, or if the `#[lsp_doc(...)]` attribute is missing.
  - **Every documented example compiles and runs** in every language that exposes it. Crate doctests run the Rust examples, and the Python (maturin wheel) and Web (Playwright) healthchecks run theirs. The iOS and Android healthchecks run each transpiled example as a test of its own: an XCTest on an iOS simulator (`platforms/swift/healthcheck/GeneratedExamples.swift`) and a JUnit test on an Android emulator (`GeneratedExamples.kt` under `androidTest`). A failing test names the example's `docs/api` page. Examples read fixtures relative to the repository root, as the doctests do. The Android test APK packs only `examples/assets/` and `path/to/`, so a new fixture goes in one of those.
  - **The website is only updated after all healthchecks pass.** The post-build website export writes MDX pages under `docs/website/src/content/docs/api/` by combining the canonical Rust docs with language-specific examples sliced from annotated healthcheck scripts. If JS/Python/Swift/Kotlin examples fail to run, the site is not regenerated.
  - These guarantees apply equally to iOS and Android — the same build-time validation that blocks a release when a JS example is missing also blocks a release when the corresponding Swift or Kotlin example is missing.
- CI gates (what must pass on PR)
  - Clippy over the whole workspace with warnings denied; rustfmt check; `cargo test` for Rust; build several example binaries; Web healthcheck (Playwright); Python wheel healthcheck; iOS healthcheck (xcodebuild on macos-15); Android healthcheck (gradle + emulator on ubuntu-latest KVM); dependency license audit (cargo-deny).
  - Each platform has its own workflow so a broken runner doesn't block the others: `.github/workflows/{pull_request,healthcheck_python,healthcheck_web,healthcheck_ios,healthcheck_android}.yml`.
- Branch naming for releases
  - In-flight version branches are named `vMAJOR.MINOR.PATCH-dev` (e.g. `v0.12.0-dev`); the GitHub Release tag drops the `-dev` suffix (e.g. `v0.12.0`). Without the suffix the branch name collides with the tag and one of them has to be renamed at release time. See `CONTRIBUTING.md` ("Starting a new version" / "Release process") for the full flow.
  - `release/vX.Y.Z` branches hold the one-commit `Package.swift` pin that the tag points at. `prepare_release.yml` force-pushes them and `post_publish_update.yml` deletes them, so don't commit to them by hand.
- Release gates
  - `prepare_release.yml` is the only way to start a release. It runs when a push to `main` changes the version in `Cargo.toml` to one with no tag, and by hand from the Actions tab. It builds the xcframework, commits the root `Package.swift` version + checksum pin on `release/vX.Y.Z`, and drafts the GitHub Release on that commit with `FragmentColorFFI.xcframework.zip` attached. Publishing the draft creates the tag there, so every tag's `Package.swift` pins the binary built from its own sources.
  - On release published (PyPI on the tag push it creates):
    - `publish_crates.yml` → crates.io.
    - `publish_npm.yml` → npm.
    - `publish_py.yml` → PyPI (wheels + sdist).
    - `publish_swift.yml` → checks the attached xcframework against the tagged pin and builds the package for the iOS Simulator from the release URL. It never rebuilds: static libraries are not bit-reproducible, so a rebuilt zip would not match the pin.
    - `publish_android.yml` → GitHub Release asset `fragmentcolor-<version>.aar` (Maven Central publishing is follow-up work).
    - `post_publish_update.yml` waits for npm and PyPI, deletes `release/vX.Y.Z`, and opens a PR that bumps consumer dependency ranges and copies the tag's `Package.swift` pin to `main`.

Module‑level invariants (authoritative AGENTS.md files)
- These short rule files are the source of truth for non‑negotiable behavior. Do not introduce code that violates them.
  - docs/api/AGENTS.md (canonical doc source: no numeric type suffixes in literals, hidden lines and per-language hidden blocks, patterns the transpiler can / can't translate, no drift tolerance)
  - src/renderer/AGENTS.md (pipelines, targets/present, MSAA/resolve, bind‑group hygiene, thin public API)
  - src/shader/AGENTS.md (mesh ownership, strict attach‑time validation, reflection/mapping order, precise errors)
  - src/mesh/AGENTS.md (schema derivation, CPU/GPU buffer packing/caching, instance handling, validation contract)
  - src/pass/AGENTS.md (role, targets, compute vs render, delegation to Shader)
  - src/texture/AGENTS.md (creation via Renderer, binding/sampling, MSAA/resolve lifecycle)

Conventions you’ll see enforced in code
- Library code avoids unwrap/expect/panic and returns typed errors (thiserror). parking_lot is used for locks. Clippy must be clean; fixers are provided.
- Public methods are intentionally thin and delegate to internal helpers; most logic sits behind re‑exports.
- Docs live in docs/api; update there first, then build to validate and to regenerate language examples/site.
- **Struct names match across every binding.** `Renderer`, `Shader`, `Pass`, `WindowTarget`, `TextureTarget`, `Size`, `ScreenRegion`, `Texture`, etc. are identical in Rust, JS, Python, Swift, and Kotlin. Achieved via `#[pyclass(name = "...")]` on Python, `wasm_bindgen(js_name = ...)` on Web, `#[cfg_attr(mobile, derive(uniffi::Object))]` on mobile, where uniffi uses the Rust type name. A mobile wrapper type takes the Rust name with `#[uniffi(name = "...")]` on the struct and `#[uniffi::export(name = "...")]` on its impl block.
- Method naming follows each language's idiom (snake_case in Rust / Python, camelCase in JS / Swift / Kotlin). The transpilers in `scripts/convert.rs` translate names automatically.
- **Qualifiers go first in names**, platform words included: `MobileTextureRegion`, never `TextureRegionMobile`. The one exception is the `_js`, `_py`, `_mobile`, `_ios` and `_android` suffix on a binding wrapper method.
- Platform-only methods (e.g. `Renderer::create_target_ios`, which Swift sees as `createTarget(metalLayerPtr:)`) live behind `#[cfg(ios)]` / `#[cfg(android)]` in a module's `platform/mobile/` folder, such as `src/renderer/platform/mobile/ios.rs`. Their `#[lsp_doc(...)]` points at a page in the object's `hidden/` folder, such as `docs/api/core/renderer/hidden/create_target_ios.md`. A Swift or Kotlin extension wraps it in the call every platform shares, such as `renderer.createTarget(layer)`.

Notes and tiny suggestions
- The per‑module AGENTS.md files are concise and clear. Keep them authoritative and link to them from PRs when changes touch those areas. Refresh the relevant module file in the same commit whenever the invariants change.

## What "looks the same" means

The API must look the same in Rust, JavaScript, Python, Swift, and Kotlin in every release. The differences below are accepted. Anything else is a gap, and a gap gets fixed in every binding at once, in the release that finds it.

1. **Case.** snake_case in Rust and Python, camelCase in JS, Swift, and Kotlin. Constants stay upper case everywhere (`Vertex.UV0`).
2. **Constructors.** `X::new(a)` in Rust is `new X(a)` in JS and `X(a)` in Python, Swift, and Kotlin. Named constructors (`from_shader`, `pbr`, `compute`, `fetch`, `load`) are static methods everywhere.
3. **Rust tuples.** When a Rust method takes one `impl Into<T>` tuple, such as `f((a, b))`, every binding takes positional arguments, `f(a, b)`. Trailing optional parts may be left out: `undefined` in JS, `=None` in Python, default arguments in Swift and Kotlin.
4. **Async.** A Rust `async fn` is a Promise in JS, `async` in Swift, and `suspend` in Kotlin, and it blocks in Python. It exists in every binding.
5. **Errors.** A Rust `Result` becomes a thrown `Error` object in JS, `FragmentColorError` or a subclass in Python (with `TypeError` only for an argument of the wrong type), `throws FragmentColorError` in Swift, and a thrown `FragmentColorException` in Kotlin. Swift's `try` is idiom. A JS `Error`'s `name` is the class Python raises for the same error, such as `FragmentColorError`, `InvalidInputError` or `LimitError`, and an argument of the wrong type throws a `TypeError` in both. Both read the class from one place, `Classify` in `src/error.rs`, so they can't drift. No binding returns a default object in place of an error, and none clamps, wraps, traps, or zero-fills invalid input.
6. **Native types.** Each binding takes its own language's forms, and a numeric literal compiles as the language writes it for the parameter's type (see rule 8).
    - Bytes: `Uint8Array` and `ArrayBuffer` in JS; `bytes`, `bytearray`, `memoryview`, and a uint8 ndarray in Python; `Data` and `[UInt8]` in Swift; `ByteArray` in Kotlin.
    - Float vectors: `number[]` and `Float32Array` in JS; any number sequence in Python except the bytes forms above, with a `bool` item raising `TypeError`; `[Float]` and `[Double]` in Swift, where a list of integer literals takes the `[Float]` overload; `List<Float>`, `FloatArray`, and `DoubleArray` in Kotlin. A color is one of these lists in every language, `[f32; 3]` or `[f32; 4]` in Rust, never a tuple or separate channel arguments. Each item of a color, a light's position or direction, an axis, a `Quad` corner, a vertex position or a UV transform is a finite number that fits a 32-bit float. NaN, an infinity or a larger number is an error that names the call and the item, by the rule in `src/platforms/vector.rs`, which the Swift and Kotlin `readFloats` repeat before a `Double` narrows to `Float`.
    - Sizes: `[w, h]` in every native list type, `[UInt32]` in Swift and `List<UInt>` in Kotlin, plus a `Size` object. Each number is a whole number, as rule 8 says.
    - Images: `get_image` returns the same bytes in the same row order everywhere. Rust, JS, Swift, and Kotlin return them flat; Python returns them as a uint8 ndarray shaped `(height, width, bytes per pixel)`, with a leading layer axis for array textures, because that is what NumPy, OpenCV, and pygfx read.
7. **Scalars.** A one-element list is a scalar everywhere. A bare number fills a float slot unless the shader declares an integer type. This holds for uniforms (`Shader::set`) and for vertex and instance attributes (`Vertex::set`, `Instance::set`), which take their type when the mesh attaches. An integer slot takes whole numbers that fit its type, and any other value is an error.
8. **Integers.** An unsigned parameter takes the language's unsigned type: `UInt32` or `UInt64` in Swift and `UInt` or `ULong` in Kotlin. A negative value doesn't compile there, so the extensions don't check for one. JavaScript takes a plain number and throws on a negative or fractional one, for sizes and regions as for counts. Python takes an int of any size, or an object with `__index__` such as a NumPy integer: a float raises `TypeError`, even `2.0`, as does a `bool`, and an int below 0 or past the type's range raises `InvalidInputError`, never `OverflowError`. Each error names the parameter, and the item for a list. A Swift literal fits as written, `pass.setComputeDispatch(64, 64, 1)`, and Kotlin writes unsigned literals, `pass.setComputeDispatch(64u, 64u, 1u)` and `listOf(64u, 64u)`. Byte lists are the exception: Swift also takes `[Int]` and Kotlin `List<Int>`, and each value must lie in 0 to 255.
9. **Paths.** A path string is the portable form. Rust also takes `Path` and `PathBuf`. Native path types (`pathlib.Path`, `URL`, `File`) are optional, and one that gets added goes into every binding at once.
10. **uniffi plumbing names.** When uniffi can't overload, a raw generated method may carry an extra name (`Shader.compose`, `Scene.loadBytes`, `Pass.setDepthTexture`, `render(renderable:target:)`). That's allowed only when all three hold: the documented spelling exists through an extension, `docs/api/PARITY` lists the alias, and no doc shows the raw name. Swift labeled raw methods next to unlabeled overloads are fine.
11. **Names.** Type names in public signatures are identical everywhere. A binding-only wrapper name (`Mobile*`, `Py*`, `RenderCanvas*`) never appears in a signature. Old names survive only as aliases.
12. **Return values.** If Rust returns `&Self` or `Self` from a mutator, every binding returns the receiver. `Option<T>` maps to `undefined`, `None`, `nil`, and `null`. Getters return the same shape everywhere: matrices in Rust's shape and sizes as `Size`, never a tagged enum.
13. **Value types** (`Size`, `ScreenRegion`, `TextureFormat`, `TextureId`, `SamplerOptions`) compare by value in Python, Swift, and Kotlin. Identity comparison in JS is accepted.
14. **Rust-only items.** Trait implementations (`SceneObject`, `Target`, `Renderable`) and the generic `get::<T>` are Rust idioms. Third-party types (glam, wgpu, naga, image, gltf, ureq, csscolorparser) never appear in a public error, Rust included: not in a variant, a field or a `From` impl. An upstream error reaches a public error as crate-owned data, such as naga's message and places in a `Diagnostic`, through a crate-private conversion, so `?` stays at the call site. The rest of the public API names none of them either: a crate type reaches its wgpu or naga counterpart through a crate-private method, such as `TextureFormat::to_wgpu` or `Size::to_extent`, never a public `From` impl, and `Renderer::create_target` takes the `raw-window-handle` traits `HasWindowHandle` and `HasDisplayHandle`, which wgpu and winit share.

### How the build checks it

- **Class names match the Rust root.** `scripts/binding_checks.rs` fails the build when the uniffi Swift wrapper, the Python stub or the wasm exports declare a class the crate root doesn't export in a plain Rust build. Name a binding class after its Rust type (`#[pyclass(name = "...")]`, `js_name`, `#[uniffi(name = "...")]`). A class that has to stay binding-only needs a `type <Name>:<platforms>:<reason>` line in the type names section at the end of `docs/api/PARITY`.
- **JS throws named errors.** Every JS error is a `WebError`, an `Error` named after the class Python raises, or a `TypeError`. A `#[wasm_bindgen]` function that returns `Result<_, E>` returns `WebError`, or an `E` whose `From<E> for JsValue` is exactly `WebError::from(error).into()`, and so is every `From<E> for JsValue` in the crate; per-variant logic goes in `From<E> for WebError`. A function that returns a `Promise` builds it with `error::defer`, whose future fails with a `WebError`, and nothing outside `src/error.rs` calls `future_to_promise`. `scripts/binding_checks.rs` fails the build otherwise.
- **Shapes match the Rust method.** `scripts/parity.rs` fingerprints each credited binding (arity, parameter classes, `Result`, `async`, return class) against the Rust method its doc page documents. A difference missing from `docs/api/PARITY_SHAPES` prints a cargo warning; the check isn't strict yet. `FC_PARITY_REWRITE_BASELINE=1 cargo build --lib` rewrites the file together with `PARITY_BASELINE`.
- **`#[doc(hidden)]` is matched exactly.** An `lsp_doc` page under `hidden/` doesn't hide a type. Mark a binding wrapper struct, such as `PyTextureTarget`, with `#[doc(hidden)]`.
- **Kotlin has no `Int` overload for an unsigned parameter.** `scripts/kotlin.rs` fails the build when a Kotlin extension takes an `Int` where the uniffi method of the same name and arity takes an unsigned integer. Delete the extension, since the generated method already takes the `UInt`.
- **The web healthcheck fails on any `console.error`.** Its `late_errors` module raises GPU errors on purpose, collects the `Device error` logs it expects, and passes every other `console.error` through.
- **The iOS, Android, and Python healthchecks fail on any uncaptured GPU error.** wgpu's error handler appends each one to the file that `FC_GPU_ERROR_LOG` names. `ExampleCase` on iOS and Android and the generated `platforms/python/examples/main.py` read that file after every test, and a failing example names its `docs/api` page.
- **CI lints the whole workspace** with `cargo clippy --workspace`, so every `fce` example has to build.

## Lock blocks (MDX) — content the user has hand-tuned

Some MDX regions are wrapped in `<Lock id="..." description="..." comments="...">…</Lock>`. **Do not edit content between an open and close `<Lock>` tag.** Don't paraphrase, don't reformat, don't "improve." The wrapping signals the user has iterated on those words and wants them preserved verbatim across agent runs.

If you genuinely think a locked region needs changing, surface it to the user in your final report rather than editing through.

How the system works (so you understand what you're respecting):

- `docs/website/integrations/locks.ts` is the Astro integration that owns the lock store. It runs in two places: `astro:server:setup` does an initial scan and then watches `docs/website/src/content/docs/**/*.{md,mdx}` via Vite's file watcher (re-scans only the changed file on save); `astro:build:start` does a full scan on production builds. Either path hashes the inner content of every `<Lock>` block (SHA-256), upserts into `docs/website/.locks/locks.json` (co-located with the website, gitignored, `chmod 600`), and bumps a per-block version when content changes. An unpaired or nested `<Lock>` fails the production build with a clear error; on the dev server the same error logs as a warning so the page still renders.
- The Rust build pipeline does NOT touch the lock store. Editing prose should never trigger a `cargo build`.
- `./fc example --release locks status | history | diff` is a thin Rust CLI that READS the JSON and surfaces history + colored diffs. Read-only.
- The hash store is convention-protected, not enforced — any process running as the user can write to it. The protection is that you (the agent) are reading this and choosing to respect the marker.

You **can** add new content **outside** locked regions, and you can re-format / restructure unlocked content normally. If you create new lock blocks of your own, that's also fine — the build script will pick them up and start a version history.
