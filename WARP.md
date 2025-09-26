# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Repository overview

- FragmentColor is a Rust library built on wgpu for cross‑platform GPU rendering, with bindings for Web (WASM/JS) and Python.
- Core Rust crate lives in src/; examples for Rust/JS/Python are under examples/ and platforms/.
- Docs: Source of truth lives in docs/api and is referenced from code via #[lsp_doc] attributes. The website (docs/website, Astro/Starlight) is generated from docs/api by build.rs; method pages embed JS/Python snippets sliced from the healthcheck scripts.
- Workspace: examples/rust (crate name fce) depends on the root library with the winit feature to run desktop demos.

## Common commands

### Rust core

- Build: cargo build
- Test (all): cargo test
- Test (single): cargo test <pattern>
- Lint (deny warnings): cargo clippy --all-targets --all-features -- -D warnings
- Format: cargo fmt

Run desktop examples (crate fce):
- cargo run -p fce --example circle
- cargo run -p fce --example triangle
- cargo run -p fce --example multiobject
- cargo run -p fce --example multipass
- Other examples available under examples/rust/examples (e.g., particles, compute_texture)

### Web (WASM)

- Build WASM package and distribute to web subprojects:
  - ./build_web [--debug]
- Run dev servers and open browser:
  - ./run_web repl        # Vite REPL (platforms/web/repl)
  - ./run_web visual      # minimal visual page (REPL server)
  - ./run_web healthcheck # Node COOP/COEP server for headless tests
  - ./run_web gallery     # visual gallery that runs all JS examples
  - Env: REPL_PORT, PORT can override default ports
- Manual alternative for simple JS demo:
  - pnpm --dir examples/javascript install && pnpm --dir examples/javascript dev

### Python binding (local dev)

- Quick run using helper (build wheel into dist/, create venv, run example):
  - ./run_py [main|multiobject|headless]
- Manual workflow:
  - pipx install maturin
  - maturin develop
  - pip install glfw rendercanvas
  - python examples/python/main.py
- Build wheel only (optional; supports macOS codesign via CODESIGN_IDENTITY):
  - ./build_py

### Docs site (Astro/Starlight)

- From docs/website:
  - pnpm install
  - pnpm dev
  - pnpm build

### Dependency/license check

- cargo deny check licenses --all-features

### Healthchecks (headless, local)

- ./healthcheck            # run Python and Web
- ./healthcheck py         # Python only
- ./healthcheck web        # Web only (uses Node + Playwright)

## High‑level architecture

### Core concepts

- Shader: Parses WGSL (and GLSL when enabled) using naga, extracts uniforms, and manages typed uniform storage. Provides set/get/list_* APIs and determines whether a shader is compute or fragment based on entry points.
- Pass: Groups one or more Shaders as a render or compute pass. Controls load vs clear behavior and optional viewport. Tracks total uniform/storage bytes required.
- Frame: An ordered collection of Passes. Implements Renderable so it can be submitted to the renderer.
- Renderer/RenderContext: Lazily initializes wgpu Instance/Adapter/Device/Queue, manages a uniform BufferPool and a TexturePool, and caches pipelines keyed by (ShaderHash, TextureFormat, SampleCount). Rendering uploads current uniform values, builds/reuses per‑group BindGroupLayouts, and draws either a fullscreen triangle or bound Meshes.
- Mesh: Holds vertex/index/instance data and schemas. Vertex buffer layouts are derived by reflecting shader inputs and mapping to mesh attributes by location and name.
- Target/TargetFrame: Abstractions over on‑screen surfaces and offscreen textures. create_target(window) prefers a Surface (WindowTarget) but falls back to a TextureTarget when surface creation fails (useful for CI/headless). create_texture_target([w, h]) yields an offscreen TextureTarget.

### Data flow (render path)

1) Shader.new(source) parses to a naga Module, validates, extracts uniforms and per‑uniform group/binding metadata; computes a ShaderHash.
2) A Pass collects shaders and declares how to load or clear the target.
3) Renderer.render(renderable, target) builds a command encoder and, per pass:
   - Ensures buffer pool capacity for uniform/storage bytes.
   - Uploads current uniform values grouped by set (group/binding); caches/reuses per‑group BindGroupLayouts.
   - Builds or fetches render/compute pipelines per (ShaderHash, Target format, SampleCount).
   - Draws a fullscreen triangle when no Mesh is attached; otherwise, binds mesh vertex/index/instance streams and draws.
4) Command buffers are submitted and the TargetFrame is presented if auto_present.

### Platforms and features

- Platform adapters live under src/**/platform and expose per‑platform entry points via cfg attributes and bindings:
  - Web: wasm-bindgen for JS/WASM; Renderer exposes createTarget and createTextureTarget.
  - Python: pyo3 for CPython extension; Python package configured via pyproject.toml with features = ["python", "pyo3/extension-module"].
- Cargo features: glsl, shadertoy (implies glsl), winit, python, fc_metrics. build.rs defines cfg aliases (wasm, ios, android, desktop, python, dev).
- Workspace includes examples/rust (crate fce) using the root library with the winit feature to run desktop demos.

### Documentation pipeline (build.rs)

- Validates docs under docs/api: ensures object and method pages exist and include a "## Example" section; enforces #[lsp_doc] coverage on public items.
- Generates a static API map (generated/api_map.rs) by scanning public items in src/; used for validation and example extraction.
- Exports website content from docs/api into docs/website (MDX), downshifting headings and escaping MDX specials; rewrites API links to canonical https://fragmentcolor.org paths.
- Slices runnable JS/Python examples from the healthcheck scripts (platforms/python/healthcheck.py and platforms/web/healthcheck/main.js) using explicit markers.

## CI overview

- Pull Request (lint, fmt, tests, build examples): .github/workflows/pull_request.yml
  - Installs headless GPU deps, runs cargo clippy, cargo fmt --check, cargo test, and builds all key Rust examples under examples/rust.
- Healthcheck Web: .github/workflows/healthcheck_web.yml
  - Builds the web WASM package and runs the Playwright headless harness against platforms/web/healthcheck.
- Healthcheck Python: .github/workflows/healthcheck_python.yml
  - Builds wheels with maturin, installs in a venv, and runs the Python healthcheck module.
- Docs CI: .github/workflows/docs-ci.yml
  - Installs docs/website deps and ensures the Astro site builds.
- Dependencies license check: .github/workflows/dependencies_check.yml (cargo deny check licenses --all-features)
- Publish to npm (tags): .github/workflows/publish_npm.yml
- Publish to PyPI (tags): .github/workflows/publish_py.yml
- Post‑publish updates: .github/workflows/post_publish_update.yml
  - Waits for registry availability, bumps example and website dependencies to the released version, and snapshots API docs into a versioned directory.

