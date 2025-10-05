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

## Release process

1. Create a version branch from `main`: `vMAJOR.MINOR.PATCH`.
2. Run `./bump_version.py` to bump versions in `Cargo.toml`, `pyproject.toml`, and the top-level version in `docs/website/package.json`.
3. Open a PR and iterate until all checks are green (lint, fmt, tests, healthchecks, docs site build). Keep `main` protected and green.
4. Merge the PR to `main`.
5. Create a GitHub Release (annotated) named `vMAJOR.MINOR.PATCH` targeting the merge commit on `main`.
6. On release published, CI will:
   - Build & publish the Web package to npm (skip if already published).
   - Build & publish Python wheels to PyPI (skip existing).
   - Publish the Rust crate to crates.io (skip if version exists).
7. The `post_publish_update` workflow waits for registries, then:
   - Bumps `fragmentcolor` dependency in `docs/website/package.json` and `examples/javascript/package.json` to the released version and refreshes lockfiles.
   - Snapshots API docs to `docs/website/src/content/docs/api/v{version}`.
   - Pushes the changes to `main`.
8. Vercel auto-deploys `docs/website` (subfolder). Its Ignored Build Step skips build when neither `docs/website` nor `docs/api` changed.

### Ad-hoc website updates

- Docs CI runs on pushes and PRs that touch `docs/website/**`.
- Heavy healthchecks are gated by paths and won’t run for docs-only changes.
- Vercel deploys automatically for `docs/website` changes and skips builds when unrelated.

### Required secrets

- `NPM_SECRET` for npm publish.
- `PYPI_API_TOKEN` for PyPI upload.
- `CARGO_REGISTRY_TOKEN` for crates.io publish.

### Tooling versions

- Node 20 and pnpm 10 for docs site builds.
- Rust toolchain stable in CI.

## Running locally

```bash
# rust
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
cargo test

# website
yarn --cwd docs/website || pnpm --dir docs/website install
pnpm --dir docs/website build
```
