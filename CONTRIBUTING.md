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

1. Create a new branch from `main`: `vMAJOR.MINOR.PATCH-dev` (the `-dev` suffix keeps the branch name out of the way of the release tag, which is `vMAJOR.MINOR.PATCH` without a suffix).
2. Run `./fc bump_version.py` to bump:
   - `Cargo.toml`
   - `pyproject.toml`
   - `docs/website/package.json` (version only)
3. Create a draft PR and implement the planned tasks (see `ROADMAP.md`).
4. Ensure all docs exist (`docs/api`), doctests pass, healthchecks are annotated, and the site builds.
5. If the uniffi surface changed (anything under `#[uniffi::export]`, or the uniffi version), run `./fc build_ios` and commit `platforms/swift/Sources/FragmentColor/generated/FragmentColor.swift`. Swift Package Manager compiles that file from the release tag, and the iOS healthcheck fails when it is stale.

## Release process

1. Create a version branch from `main`: `vMAJOR.MINOR.PATCH-dev`. The `-dev` suffix keeps the branch out of the way of the release tag created in step 6. Without it, the branch and tag share a name and one of them has to be renamed at release time.
2. Run `./fc bump_version.py` to bump versions in `Cargo.toml`, `pyproject.toml`, and the top-level version in `docs/website/package.json`.
   It should also refresh launch-facing version references such as the root README, Python example requirements, and the website version badge.
3. Open a PR and iterate until all checks are green (lint, fmt, tests, healthchecks, docs site build). Keep `main` protected and green.
4. Merge the PR to `main`.
5. Run the **Prepare release** workflow (`prepare_release.yml`) on `main`, from the Actions tab or with `gh workflow run prepare_release.yml --ref main`. It:
   - Builds the iOS **xcframework** from `main`, and fails if the committed Swift wrapper differs from what `./fc build_ios` generates.
   - Commits the root `Package.swift` pin (`fragmentcolorVersion` / `fragmentcolorChecksum`) on top of that commit and pushes it to `release/vMAJOR.MINOR.PATCH`.
   - Creates a draft GitHub Release `vMAJOR.MINOR.PATCH` that targets the pinned commit, with `FragmentColorFFI.xcframework.zip` attached. The notes are the `CHANGELOG.md` entry followed by GitHub's generated list of changes.

   Running it again replaces the binary, the pin commit, and the draft's target, and keeps the draft's notes.
6. Review the draft and publish it. Publishing creates the `vMAJOR.MINOR.PATCH` tag on the pinned commit, one commit ahead of the merge commit on `main`. Swift Package Manager resolves `from: "X.Y.Z"` to that tag, so consumers get the xcframework built from the tag's own sources. Don't create the release by hand: a tag without the pin ships the previous release's binary next to the new Swift wrapper.
7. On release published, CI will (in parallel):
   - Build & publish the Web package to npm (skip if already published).
   - Build & publish Python wheels to PyPI (skip existing). This workflow runs on the tag push.
   - Publish the Rust crate to crates.io (skip if version exists).
   - Check the attached xcframework against the tagged `Package.swift` pin and build the Swift package for the iOS Simulator from the release URL (`publish_swift.yml`). It never rebuilds the xcframework, because static libraries are not bit-reproducible and a rebuild would not match the pin.
   - Build the Android **AAR** (all 4 ABIs) and upload `fragmentcolor-<version>.aar` to the GitHub Release as an asset.
8. The `post_publish_update` workflow waits for npm and PyPI to land, then:
   - Bumps `fragmentcolor` dependency in `docs/website/package.json` and `examples/javascript/package.json` to the released version and refreshes lockfiles.
   - Updates `docs/website/src/components/VersionBadge.astro`.
   - Copies the `Package.swift` pin from the tag, so `main` points at the latest release's xcframework.
   - Deletes the `release/vMAJOR.MINOR.PATCH` branch. The tag keeps the pinned commit.
   - Opens a pull request with those consumer updates.
9. Vercel auto-deploys `docs/website` (subfolder). Its Ignored Build Step skips build when neither `docs/website` nor `docs/api` changed.
10. Swift consumers add `.package(url: "https://github.com/vista-art/fragmentcolor", from: "X.Y.Z")` to their dependencies. SPM resolves the matching tag and downloads the xcframework that the tag's `Package.swift` pins.
11. Kotlin / Android consumers: for 0.11.x, download `fragmentcolor-<version>.aar` from the GitHub Release and drop it into `libs/`; Maven Central publishing is tracked as follow-up work (CHANGELOG "Unfinished work").

### Ad-hoc website updates

- Docs CI runs on pushes and PRs that touch `docs/website/**`.
- Heavy healthchecks are gated by paths and won’t run for docs-only changes.
- Vercel deploys automatically for `docs/website` changes and skips builds when unrelated.

### Required secrets

- `NPM_SECRET` for npm publish.
- `PYPI_API_TOKEN` for PyPI upload.
- `CARGO_REGISTRY_TOKEN` for crates.io publish.
- The prepare, Swift, and Android workflows use the default `GITHUB_TOKEN` for the release branch, the draft release, and asset uploads. No extra secret is needed until Maven Central support lands.

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
