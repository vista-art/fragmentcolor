# Renderer::create_texture_target_mobile()

Concrete-typed variant of `create_texture_target` used by uniffi (Swift /
Kotlin). Uniffi cannot marshal `impl Into<Size>`, so the mobile entry point
takes the `Size` record, layers included, and checks it the way every binding
does.

The Swift and Kotlin extensions add the `[width, height]` list overload, and
Swift adds a `Size` one, matching the JS and Python spelling.

## Example

```rust
// hidden file; no public example
```
