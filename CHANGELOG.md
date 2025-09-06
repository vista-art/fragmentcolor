# Changelog

## Unreleased

See the [Roadmap](https://github.com/vista-art/fragmentcolor/blob/main/ROADMAP.md) for planned features.

## 0.10.7 Documentation automation, website integration, and release flow

Added
- Automated documentation pipeline:
  - Doc strings centralized in `docs/api` and consumed in Rust via `#[lsp_doc]`.
  - Build-time validation: ensures object/method docs exist and include a `## Example` section.
  - Website generator: converts `docs/api` into MDX pages under `docs/website`, downshifting method headings and stripping object H1.
  - JS/Python examples are sliced from annotated healthcheck scripts.
- Website moved into this repository under `docs/website`.
- Post-publish workflow: after tags publish to npm & PyPI, update consumers (website & JS example) to the released version and push to main.
- Healthcheck example markers added for Renderer/Pass/Frame/Shader.

Changed
- Normalized API links to https://fragmentcolor.org.
- Wired all public items to `#[lsp_doc]` sources (Renderer, Shader, Pass, Frame, etc.).

## 0.10.6 JavaScript support

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
