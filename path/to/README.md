# Static `Scene::load` fixtures

These files exist so the `Scene::load` documentation examples, which use the
literal placeholder path `path/to/model.glb` (and `.gltf`), resolve against a
real file when the doctests and the Python healthcheck actually run them.

- `model.glb` / `model.gltf`: a minimal 3-vertex (positions-only) glTF 2.0
  triangle. Both encode the same geometry; the `.gltf` variant inlines the
  buffer as a base64 data URI so it needs no sibling `.bin`.

They are committed static assets, not generated at build time. The reference
builder for the same triangle lives in the `Scene::load` integration tests at
`src/scene/loader.rs` if the format ever needs to change. The web healthcheck
serves its own copy at `platforms/web/healthcheck/public/model.glb`, fetched by
the JS `Scene.load` override.
