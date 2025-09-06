# Contributing

Contributions are welcome! This document outlines the current workflow for development, docs, and releases.

## Documentation workflow

- Source of truth for docs is `docs/api`. Public objects and methods must have:
  - Object page: `docs/api/{object}/{object}.md` with `## Example`.
  - Per-method page: `docs/api/{object}/{method}.md` with `## Example`.
- Rust code uses `#[lsp_doc("docs/api/..."))]` to pull docs into editor hovers and unify docs across bindings.
- The website is generated from `docs/api` at build time into `docs/website/src/content/docs/api/*.mdx`.
- JavaScript & Python examples on the website are extracted from healthchecks:
  - platforms/web/healthcheck/main.js
  - platforms/python/healthcheck.py
  - Use markers like:
    - JS: `// DOC: Renderer.create_texture_target (begin)` ... `// DOC: (end)`
    - Py: `# DOC: Renderer.create_texture_target (begin)` ... `# DOC: (end)`

## Conventions

- Prefer parking_lot alternatives over std Mutex/RwLock.
- Prefer short, descriptive names; start function names with verbs.
- Prefer named lifetimes over single-letter.
- No unwrap/expect/panic in library code; use `thiserror` for libraries, `anyhow` for apps.
- Run `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` before pushing.

## Starting a new version

1. Create a new branch from `main`: `vMAJOR.MINOR.PATCH`.
2. Run `./bump_version.py` to bump:
   - `Cargo.toml`
   - `pyproject.toml`
   - `docs/website/package.json` (version only)
3. Create a draft PR and implement the planned tasks (see `ROADMAP.md`).
4. Ensure all docs exist (`docs/api`), doctests pass, healthchecks are annotated, and the site builds.

## Publishing

- Tag the release on the release branch (e.g., `v0.10.7`).
  - This triggers npm and PyPI publish workflows.
- After publish, the `post_publish_update` workflow waits for packages to become available and updates:
  - `docs/website/package.json` dependencies.fragmentcolor
  - `examples/javascript/package.json` dependencies.fragmentcolor
  - Then pushes the changes to `main`.
- Finally, merge the release branch to `main`.

## Running locally

```bash
# rust
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
cargo test

# website
yarn --cwd docs/website || pnpm --dir docs/website install
pnpm --dir docs/website build
```
