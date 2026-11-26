# Shader::getData (mobile)

Mobile binding for `Shader::get`.
Uniffi can't return one of several types, so the raw method returns the
`UniformData` enum, with a struct, array or buffer root flattened to its bytes.
The Swift and Kotlin extensions expose it as `get(key)`, which returns the
plain value, as JavaScript and Python do.

## Example

```rust
// hidden mobile binding; no public example
```
