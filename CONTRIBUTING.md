# Contributing

Contributions are welcome! This document outlines the current workflow for development, docs, and releases.

## Documentation workflow

- Source of truth for docs is `docs/api`. Public objects and methods must have:
  - Object page: `docs/api/{object}/{object}.md` with `## Example`.
  - Per-method page: `docs/api/{object}/{method}.md` with `## Example`.
- Rust code uses `#[lsp_doc("docs/api/..."))]` to pull docs into editor hovers and unify docs across bindings.
- The website is generated from `docs/api` at build time into `docs/website/src/content/docs/api/*.mdx`.
- JavaScript, Python, Swift, and Kotlin examples on the website are extracted from healthchecks:
  - platforms/web/healthcheck/main.js
  - platforms/python/healthcheck.py
  - platforms/swift/healthcheck/Healthcheck.swift
  - platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/Healthcheck.kt
  - Use markers like:
    - JS: `// DOC: Renderer.create_texture_target (begin)` ... `// DOC: (end)`
    - Py: `# DOC: Renderer.create_texture_target (begin)` ... `# DOC: (end)`
    - Swift / Kotlin: `// DOC: Renderer.createTextureTarget (begin)` ... `// DOC: (end)`
- Swift and Kotlin example files under `platforms/{swift,kotlin}/examples/` are auto-generated
  from the Rust `## Example` snippets via `scripts/swift.rs` and `scripts/kotlin.rs`. Do not
  hand-edit those files — update the source `docs/api/**/*.md` and rerun `cargo build`.

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
   It should also refresh launch-facing version references such as the root README, Python example requirements, and the website version badge.
3. Open a PR and iterate until all checks are green (lint, fmt, tests, healthchecks, docs site build). Keep `main` protected and green.
4. Merge the PR to `main`.
5. Create a GitHub Release (annotated) named `vMAJOR.MINOR.PATCH` targeting the merge commit on `main`.
6. On release published, CI will (in parallel):
   - Build & publish the Web package to npm (skip if already published).
   - Build & publish Python wheels to PyPI (skip existing).
   - Publish the Rust crate to crates.io (skip if version exists).
   - Build the iOS **xcframework** and upload `fragmentcolor.xcframework.zip` to the GitHub Release as an asset.
   - Build the Android **AAR** (all 4 ABIs) and upload `fragmentcolor-<version>.aar` to the GitHub Release as an asset.
7. The `post_publish_update` workflow waits for npm, PyPI **and** the xcframework asset to land, then:
   - Bumps `fragmentcolor` dependency in `docs/website/package.json` and `examples/javascript/package.json` to the released version and refreshes lockfiles.
   - Updates `docs/website/src/components/VersionBadge.astro`.
   - Pins the root-level `Package.swift` `fragmentcolorVersion` / `fragmentcolorChecksum` to the just-published xcframework so Swift Package Manager consumers on `from: "X.Y.Z"` get a matching binary.
   - Opens a pull request with those consumer updates.
8. Vercel auto-deploys `docs/website` (subfolder). Its Ignored Build Step skips build when neither `docs/website` nor `docs/api` changed.
9. Swift consumers: `dependencies: [.package(url: "https://github.com/vista-art/fragmentcolor", from: "X.Y.Z")]` — SPM resolves to the matching tag and downloads the xcframework release asset.
10. Kotlin / Android consumers: for 0.11.x, download `fragmentcolor-<version>.aar` from the GitHub Release and drop it into `libs/`; Maven Central publishing is tracked as follow-up work (CHANGELOG "Unfinished work").

### Ad-hoc website updates

- Docs CI runs on pushes and PRs that touch `docs/website/**`.
- Heavy healthchecks are gated by paths and won’t run for docs-only changes.
- Vercel deploys automatically for `docs/website` changes and skips builds when unrelated.

### Required secrets

- `NPM_SECRET` for npm publish.
- `PYPI_API_TOKEN` for PyPI upload.
- `CARGO_REGISTRY_TOKEN` for crates.io publish.
- Swift / Android publish workflows use the default `GITHUB_TOKEN` for release-asset uploads — no extra secret needed until Maven Central support lands.

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
