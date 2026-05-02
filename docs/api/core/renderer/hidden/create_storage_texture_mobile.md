# Renderer::createStorageTexture

Mobile (Swift / Kotlin via uniffi) wrapper for `Renderer::create_storage_texture`. Takes the fields directly because uniffi cannot marshal `impl Into<TextureInput>`; the body builds a `TextureInput` internally (using `TextureData::Empty` when `data` is omitted, or `TextureData::Bytes(...)` when pre-seeding). `usage_bits = nil` defaults to `STORAGE | TEXTURE | COPY_SRC | COPY_DST`; pass `data` to pre-seed the texture, or omit it for an empty allocation.

## Example

```rust
// hidden file; canonical example lives in create_storage_texture.md
```
