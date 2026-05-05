# Renderer::createTexture

Mobile (Swift / Kotlin via uniffi) wrapper for `Renderer::create_texture`. Takes a `TextureInputMobile` enum + an optional `TextureOptions` because uniffi cannot marshal `impl Into<TextureInput>`. Swift / Kotlin extension files supply the natural overloads (e.g. `renderer.createTexture(bytes)` or `renderer.createTexture(chain)`) by wrapping the enum invisibly so users never see the mobile-only mirror type.

## Example

```rust
// hidden file; canonical example lives in create_texture.md
```
