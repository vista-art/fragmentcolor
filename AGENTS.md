# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

Project summary
- Crate: fragmentcolor (library) with examples in examples/rust (crate fce).
- Languages: Rust core with generated bindings/examples for Web (WASM/JS) and Python.
- Toolchain: rust-toolchain set to stable with clippy/rustfmt and common cross targets.

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
- Web (WASM)
  - Build pkg: `./build_web` (debug: `./build_web --debug`)
  - Dev servers: `./run_web [gallery|repl|visual]` (respects `PORT`, `REPL_PORT`)
  - Sync JS deps: `./sync_js`
- Python
  - Local run helper: `./run_py [main|multiobject|headless]` (builds wheel, manages venv)
  - Manual build: `pipx install maturin && ./build_py`
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
- Documentation‑driven pipeline
  - Canonical docs live in docs/api and are pulled into Rust via #[lsp_doc(...)] so IDE hovers match the website.
  - build.rs enforces a “no‑panic in library code” policy, scans the public API to generate generated/api_map.rs and language examples, writes healthcheck aggregators, and exports website pages.
  - Tests include crate doctests and harness checks that validate the generated JS/Python examples (e.g., string formatting and WGSL integrity) to keep docs/examples and code in lockstep.
- CI gates (what must pass on PR)
  - Clippy with warnings denied; rustfmt check; `cargo test` for Rust; build several example binaries; Web healthcheck (Playwright) and Python wheel healthcheck; dependency license audit (cargo‑deny). Release jobs publish to crates.io, npm, and PyPI.

Module‑level invariants (authoritative WARP.md files)
- These short rule files are the source of truth for non‑negotiable behavior. Do not introduce code that violates them.
  - src/renderer/WARP.md (pipelines, targets/present, MSAA/resolve, bind‑group hygiene, thin public API)
  - src/shader/WARP.md (mesh ownership, strict attach‑time validation, reflection/mapping order, precise errors)
  - src/mesh/WARP.md (schema derivation, CPU/GPU buffer packing/caching, instance handling, validation contract)
  - src/pass/WARP.md (role, targets, compute vs render, delegation to Shader)
  - src/frame/WARP.md (DAG semantics, presentation rules, error surface)
  - src/texture/WARP.md (creation via Renderer, binding/sampling, MSAA/resolve lifecycle)

Conventions you’ll see enforced in code
- Library code avoids unwrap/expect/panic and returns typed errors (thiserror). parking_lot is used for locks. Clippy must be clean; fixers are provided.
- Public methods are intentionally thin and delegate to internal helpers; most logic sits behind re‑exports.
- Docs live in docs/api; update there first, then build to validate and to regenerate language examples/site.

Notes and tiny suggestions
- The per‑module WARP.md files are concise and clear. Keep them authoritative and link to them from PRs when changes touch those areas. No immediate revisions needed.
