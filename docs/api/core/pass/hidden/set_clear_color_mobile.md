# Pass::setClearColor (mobile)

Mobile binding for `Pass::set_clear_color`. Accepts a `Vec<Float>` of 3 or 4
linear components (`[r, g, b]` or `[r, g, b, a]`) and passes each one to the GPU
as given. A wrong length or a channel that is NaN or infinite throws.

Uniffi cannot marshal `[f32; 4]` fixed-length arrays across the FFI boundary,
so the mobile shim takes a `Vec<Float>` and validates the length at runtime.

The Swift extension also takes `[Double]`, and the Kotlin extensions take
`FloatArray`, `DoubleArray` and `List<Double>`, the vector forms every other
color setter takes. A color is a list in every language, never separate channels.

## Example

```rust
// hidden mobile binding; no public example
```
