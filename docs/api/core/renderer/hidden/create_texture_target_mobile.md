# Renderer::create_texture_target_mobile()

Concrete-typed variant of `create_texture_target` used by uniffi (Swift /
Kotlin). Uniffi cannot marshal `impl Into<Size>`, so the mobile entry point
accepts width and height as `u32` primitives and constructs the `Size`
internally.

Swift/Kotlin extensions wrap this with a single `createTextureTarget(size)`
overload matching the JS / Python spelling.

## Example

```rust
// hidden file; no public example
```
