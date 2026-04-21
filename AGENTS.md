# AGENTS.md

This file is the top-level guide for any AI coding agent (Claude Code, WARP, Cursor, Copilot CLI, …) working in this repository. Per-module rules live in sibling `AGENTS.md` files (see the "Module-level invariants" section below).

Project summary
- Crate: fragmentcolor (library) with examples in examples/rust (crate fce).
- Languages: Rust core with generated bindings/examples for Web (WASM/JS), Python, Swift (iOS), and Kotlin (Android).
- Toolchain: rust-toolchain set to stable with clippy/rustfmt and common cross targets (desktop, wasm, iOS aarch64 + sim, all 4 Android ABIs).

Common commands
- Rust core
  - Build: `cargo build`
  - Test (workspace): `./test` (runs clippy, all Rust tests, and crate doctests)
  - Test (single):
    - Unit/integration by name: `cargo test -p fragmentcolor <name> -- --exact --nocapture`
    - Specific integration file: `cargo test -p fragmentcolor --test e2e <name>`
  - Examples: interactive runner `./example`, or directly `cargo run -p fce --example triangle`
- Lint & format
  - Fast check: `cargo clippy --all-targets --all-features -- -D warnings`
  - Preferred flow: `./clippy` (formats first); autofix: `./clippy fix`
- Cross‑platform healthchecks (generated samples)
  - All: `./healthcheck`
  - Web only: `./healthcheck web` (uses Playwright; respects `PORT`, `FC_HEALTHCHECK_VERBOSE=1`)
  - Python only: `./healthcheck py`
  - iOS only: `./healthcheck ios` (runs Swift examples against the generated xcframework on an iOS simulator — requires Xcode)
  - Android only: `./healthcheck android` (runs Kotlin examples against the generated AAR on an Android emulator — requires cargo-ndk + Android SDK + emulator image)
- Web (WASM)
  - Build pkg: `./build_web` (debug: `./build_web --debug`)
  - Dev servers: `./run_web [gallery|repl|visual]` (respects `PORT`, `REPL_PORT`)
  - Sync JS deps: `./sync_js`
- Python
  - Local run helper: `./run_py [main|multiobject|headless]` (builds wheel, manages venv)
  - Manual build: `pipx install maturin && ./build_py`
- iOS (Swift)
  - Build xcframework + regenerate Swift bindings: `./build_ios`
  - Produces `build/ios/fragmentcolor.xcframework` and Swift sources under `platforms/swift/Sources/FragmentColor/generated/`
  - Example app: `platforms/swift/examples/` (opened via Xcode)
- Android (Kotlin)
  - Build `.so` per ABI + regenerate Kotlin bindings: `./build_android [TARGET]`
  - Requires: `cargo install cargo-ndk` and `ANDROID_NDK_HOME` pointing at a valid NDK
  - Produces `jniLibs/<abi>/libfragmentcolor.so` plus Kotlin sources under `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/generated/`
  - Example app: `platforms/kotlin/examples/`
- Docs site (Astro/Starlight)
  - Dev: `./run_docs` (port via `--port` or `DOCS_PORT`)
  - Preview (built): `./run_docs preview`

Big‑picture architecture (how things fit together)
- Public API surface (src/lib.rs)
  - Core types are re‑exported at the crate root: Renderer, Shader, Pass, Frame, Texture, Target (WindowTarget/TextureTarget), Mesh, Vertex, Size, Color, Region, plus stable kind branding via fc_kind.
  - Platform shims live under each module’s platform/ submodule (python, web, winit, etc.).
- Renderer and context
  - Renderer lazily creates a RenderContext (wgpu::Device/Queue). It caches render/compute pipelines keyed by descriptive structs; manages uniform/storage/readback buffer pools and a small texture pool for transient MSAA.
  - create_target prefers a surface; on failure it logs and falls back to an offscreen TextureTarget so headless/CI runs still work. MSAA for surface targets uses transient textures resolved into the swapchain view.
  - Uniforms/textures are reflected from Shader storage. Textures are registered in a Renderer‑owned registry and referenced by an integer TextureId.
  - Push constants: native push constants are used when available; otherwise they are lowered to a fallback uniform buffer per root on platforms without push‑constant support (e.g., Web).
- Shader, Mesh, Pass, Frame
  - Shader owns attached meshes and validates compatibility at attach time (formats/locations must match exactly). Fullscreen shaders (no @location vertex inputs) reject mesh attachments.
  - Pass is a thin orchestrator for shaders and render‑time knobs (viewport, clear color, compute dispatch, optional per‑pass targets). A pass is compute if all attached shaders are compute.
  - Frame is a small DAG over passes; edges encode ordering; one render leaf is selected to present. Execution uses a topo sort; cycles/invalid refs become clear errors.
- Documentation‑driven pipeline (THIS IS LOAD-BEARING)
  - Canonical docs live in docs/api and are pulled into Rust via #[lsp_doc(...)] so IDE hovers match the website.
  - build.rs enforces a “no‑panic in library code” policy, scans the public API to generate generated/api_map.rs and language examples, writes healthcheck aggregators, and exports website pages.
  - Tests include crate doctests and harness checks that validate the generated JS/Python examples (e.g., string formatting and WGSL integrity) to keep docs/examples and code in lockstep.
  - **Invariant:** any public item (object or method) visible to at least one binding must have a docs/api page with a `## Example` section, an `#[lsp_doc(...)]` attribute on the Rust item, and a working transpiled example for every language that exposes it.
- Cross-language guarantees (the whole point)
  - **Documentation is always up to date** and derived from docs/api — never hand-edit the website, healthcheck examples, or platform-specific docstrings. The website is generated from docs/api; the healthcheck examples are generated from docs/api; Rust hover docs are pulled from docs/api.
  - **100% API parity is enforced at build time** across Rust, Python, JavaScript, Swift, and Kotlin. `build.rs` fails if a public object exists in the Rust API without a doc page, if a method lacks `## Example`, or if the `#[lsp_doc(...)]` attribute is missing.
  - **Every documented example compiles and runs** in every language that exposes it. The per-language healthchecks (Python maturin wheel, Web via Playwright, Swift via xcodebuild+simulator, Kotlin via gradle+emulator) execute the transpiled examples and assert they produce the same output.
  - **The website is only updated after all healthchecks pass.** The post-build website export writes MDX pages under `docs/website/src/content/docs/api/` by combining the canonical Rust docs with language-specific examples sliced from annotated healthcheck scripts. If JS/Python/Swift/Kotlin examples fail to run, the site is not regenerated.
  - These guarantees apply equally to iOS and Android — the same build-time validation that blocks a release when a JS example is missing also blocks a release when the corresponding Swift or Kotlin example is missing.
- CI gates (what must pass on PR)
  - Clippy with warnings denied; rustfmt check; `cargo test` for Rust; build several example binaries; Web healthcheck (Playwright); Python wheel healthcheck; iOS healthcheck (xcodebuild on macos-14); Android healthcheck (gradle + emulator on ubuntu-latest KVM); dependency license audit (cargo-deny).
  - Each platform has its own workflow so a broken runner doesn't block the others: `.github/workflows/{pull_request,healthcheck_python,healthcheck_web,healthcheck_ios,healthcheck_android}.yml`.
- Release gates (what happens on tag published)
  - `publish_crates.yml` → crates.io.
  - `publish_npm.yml` → npm.
  - `publish_py.yml` → PyPI (wheels + sdist).
  - `publish_swift.yml` → GitHub Release asset `fragmentcolor.xcframework.zip` (SPM consumes it via the root `Package.swift` binaryTarget URL + checksum).
  - `publish_android.yml` → GitHub Release asset `fragmentcolor-<version>.aar` (Maven Central publishing is follow-up work).
  - `post_publish_update.yml` waits for npm, PyPI, and the xcframework asset, then opens a PR that bumps consumer dependency ranges and pins `Package.swift` to the matching checksum.

Module‑level invariants (authoritative AGENTS.md files)
- These short rule files are the source of truth for non‑negotiable behavior. Do not introduce code that violates them.
  - src/renderer/AGENTS.md (pipelines, targets/present, MSAA/resolve, bind‑group hygiene, thin public API)
  - src/shader/AGENTS.md (mesh ownership, strict attach‑time validation, reflection/mapping order, precise errors)
  - src/mesh/AGENTS.md (schema derivation, CPU/GPU buffer packing/caching, instance handling, validation contract)
  - src/pass/AGENTS.md (role, targets, compute vs render, delegation to Shader)
  - src/frame/AGENTS.md (DAG semantics, presentation rules, error surface)
  - src/texture/AGENTS.md (creation via Renderer, binding/sampling, MSAA/resolve lifecycle)

Conventions you’ll see enforced in code
- Library code avoids unwrap/expect/panic and returns typed errors (thiserror). parking_lot is used for locks. Clippy must be clean; fixers are provided.
- Public methods are intentionally thin and delegate to internal helpers; most logic sits behind re‑exports.
- Docs live in docs/api; update there first, then build to validate and to regenerate language examples/site.
- **Struct names match across every binding.** `Renderer`, `Shader`, `Pass`, `Frame`, `WindowTarget`, `TextureTarget`, `Size`, `Region`, `Texture`, etc. are identical in Rust, JS, Python, Swift, and Kotlin. Achieved via `#[pyclass(name = "...")]` on Python, `wasm_bindgen(js_name = ...)` on Web, `#[cfg_attr(mobile, derive(uniffi::Object))]` (uniffi uses the Rust type name verbatim — no renaming needed).
- Method naming follows each language's idiom (snake_case in Rust / Python, camelCase in JS / Swift / Kotlin) — the transpilers in `scripts/convert.rs` translate names automatically.
- Mobile-specific methods (e.g. `Renderer.from_metal_layer` on iOS) live behind `#[cfg(ios)]` / `#[cfg(android)]` in `src/platforms/mobile/` and still require a `docs/api/core/renderer/from_metal_layer.md` page with `## Example` blocks in Rust, Swift, and Kotlin.

Notes and tiny suggestions
- The per‑module AGENTS.md files are concise and clear. Keep them authoritative and link to them from PRs when changes touch those areas. Refresh the relevant module file in the same commit whenever the invariants change.
