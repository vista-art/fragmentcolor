# MaterialMap::source (mobile)

Mobile binding for `MaterialMap::texture` with a texture input and its options:
bytes, raw pixels with their size, a path, a URL or a mipmap chain, as `createTexture` takes them.
Uniffi cannot marshal `impl Into<TextureInput>` and has no method overloading,
so the input form gets its own entry point next to the `Texture` and map ones.
The Swift and Kotlin extensions expose it as the `texture` overloads, which match `createTexture`'s.

## Example

```rust
// hidden mobile binding; no public example
```
