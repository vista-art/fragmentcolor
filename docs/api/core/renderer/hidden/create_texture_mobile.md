# Renderer::createTexture

Mobile (Swift / Kotlin via uniffi) wrapper for `Renderer::create_texture`. Takes a `TextureData` enum, the uniffi mirror of the Rust one without `Empty`, and an optional `TextureOptions`, because uniffi cannot marshal `impl Into<TextureInput>`. Swift / Kotlin extension files supply the natural overloads (e.g. `renderer.createTexture(bytes)` or `renderer.createTexture(chain)`) by wrapping the enum, so no example names it.

## Example

```rust
// hidden file; canonical example lives in create_texture.md
```
