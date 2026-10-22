# Material::normalSource (mobile)

Mobile binding for `Material::normal_texture` with a texture input: bytes, a path, a URL or a mipmap chain.
Uniffi cannot marshal `impl Into<TextureInput>` and has no method overloading,
so the input form gets its own entry point next to the `Texture` one.
The Swift and Kotlin extensions expose it as `normalTexture(source)`.

## Example

```rust
// hidden mobile binding; no public example
```
