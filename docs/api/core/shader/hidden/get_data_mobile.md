# Shader::getData (mobile)

Mobile binding for `Shader::get`.
Uniffi can't return one of several types, so the raw method returns a
`UniformData` enum with one case per plain value: a scalar, a list of floats,
unsigned or signed integers, a texture id, a sampler, or bytes for a struct,
array or buffer root.
The Swift and Kotlin extensions expose it as `get(key)`, which returns the
plain value, as JavaScript and Python do.

## Example

```rust
// hidden mobile binding; no public example
```
