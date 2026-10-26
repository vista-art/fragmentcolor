# Pass::setClearColor (mobile)

Mobile binding for `Pass::set_clear_color`. Accepts a `Vec<Float>` of 3 or 4
linear components (`[r, g, b]` or `[r, g, b, a]`) and passes each one to the GPU
as given. A wrong length or a channel that is NaN or infinite throws.

Uniffi cannot marshal `[f32; 4]` fixed-length arrays across the FFI boundary,
so the mobile shim takes a `Vec<Float>` and validates the length at runtime.

Swift / Kotlin extension shims provide typed overloads
(`pass.setClearColor(r:g:b:)` and `pass.setClearColor(r:g:b:a:)`) so callers
never need to build a list manually.

## Example

```rust
// hidden mobile binding; no public example
```
