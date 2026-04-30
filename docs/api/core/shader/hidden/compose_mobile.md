# Shader::compose_mobile()

Uniffi-friendly array form of the constructor: takes a `Vec<String>` and
runs every entry through the same input classifier as `Shader::new`.
Each entry can be a raw shader source, a registry slug like
`"sdf2d/circle"`, an `https://` URL, or a local path.

Swift / Kotlin extensions re-expose this as `Shader([…])` so the array
form looks identical to the Rust, JS and Python composition syntax.

## Example

```rust
// hidden file; no public example
```
