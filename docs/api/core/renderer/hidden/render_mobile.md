# Renderer::render

Mobile (Swift / Kotlin via uniffi) wrapper for `Renderer::render`. Takes the renderable as a `RenderableHandle` enum (Shader / Pass / Mesh / Passes) and the target as a `TargetHandle` enum (Window / Texture) because uniffi cannot marshal `&impl Renderable` / `&impl Target`. The Swift and Kotlin extension files add overloads (`renderer.render(shader, target)`, `renderer.render(pass, target)`, etc.) that wrap the concrete value into the matching enum, so the binding-level enum types stay out of the public surface.

## Example

```rust
// hidden file; canonical example lives in render.md
```
