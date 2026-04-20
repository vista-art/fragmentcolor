# Shader::new_mobile()

Uniffi-friendly constructor taking an owned `String` (uniffi always marshals
strings by value; the core `Shader::new(&str)` stays untouched for Rust
users).

Swift / Kotlin extensions re-expose this as `Shader(source)` for idiomatic
construction that matches Rust, JS and Python.

## Example

```rust
// hidden file; no public example
```
