# Material::metallicRoughnessSource (mobile)

Mobile binding for `Material::metallic_roughness_texture` with a texture input and its options:
bytes, raw pixels with their size, a path, a URL or a mipmap chain, as `createTexture` takes them.
Uniffi cannot marshal `impl Into<TextureInput>` and has no method overloading,
so the input form gets its own entry point next to the `Texture` one.
The Swift and Kotlin extensions expose it as the `metallicRoughnessTexture` overloads, which match `createTexture`'s.

## Example

```rust
// hidden mobile binding; no public example
```
