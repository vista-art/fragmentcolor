# Pass::setTargetTexture (mobile)

Mobile binding for `Pass::set_target` with a `Texture`.
Uniffi cannot marshal `impl TryInto<ColorTarget>` and has no method overloading,
so the `Texture` form gets its own entry point.
The Swift and Kotlin extensions expose it as `setTarget(texture)`.

## Example

```rust
// hidden mobile binding; no public example
```
