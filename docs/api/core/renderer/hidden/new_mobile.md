# Renderer::new_mobile()

Uniffi-friendly constructor exposed to Swift (iOS) and Kotlin (Android).
Returns `Arc<Renderer>` because every uniffi object is reference-counted
across the FFI boundary. uniffi makes a constructor named `new` the primary
constructor, so Swift and Kotlin both call `Renderer()`.

Hidden from public website; Rust users continue to call `Renderer::new()`.

## Example

```rust
// hidden file; no public example
```
