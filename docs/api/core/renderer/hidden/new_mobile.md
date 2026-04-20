# Renderer::new_mobile()

Uniffi-friendly constructor exposed to Swift (iOS) and Kotlin (Android).
Returns `Arc<Renderer>` because every uniffi object is reference-counted
across the FFI boundary. Swift/Kotlin extension files re-expose this as
the natural `Renderer()` default initializer.

Hidden from public website; Rust users continue to call `Renderer::new()`.

## Example

```rust
// hidden file; no public example
```
