# Refactor plan tracking (WGPU hardening inspired by Ruffle)

This document tracks a multi-session refactor plan to adopt proven patterns from Ruffle’s wgpu backend.

We keep a local copy of Ruffle’s wgpu crate for reference under `ruffle_reference/wgpu`.

Tasks (click to see details):

- [x] [01: Surface configuration selection and view_formats](refactor/01_surface_config.md)
- [x] [02: Surface frame acquire recovery (WindowTarget)](refactor/02_surface_recovery.md)
- [x] [03: Sample-count negotiation helper](refactor/03_sample_count.md)
- [x] [04: Store and propagate sample_count in RenderContext](refactor/04_rendercontext_sample_count.md)
- [x] [05: Pipeline cache keyed by (ShaderHash, format, samples)](refactor/05_pipeline_cache_key.md)
- [ ] [06: MSAA render path with resolve](refactor/06_msaa_resolve.md)
- [ ] [07: Centralized frame acquire retry in Renderer (optional)](refactor/07_renderer_retry.md)
- [ ] [08: Pooling for transient targets/readback (optional)](refactor/08_pooling.md)
- [ ] [09: Docs + clippy + fmt sweep](refactor/09_docs_lint.md)
- [ ] [10: Tests and examples validation](refactor/10_tests_examples.md)

Notes
- “Done” means implemented and committed in this repo.
- Focus each task on relevant files; cross-reference specific files in `ruffle_reference/wgpu` that demonstrate the approach.
