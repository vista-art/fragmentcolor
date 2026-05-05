# Pass::addTarget (mobile)

Mobile binding for `Pass::add_target`. Accepts a `TargetHandle::Texture`
variant; window targets are rejected with an error because they cannot be used
as render-to-texture colour targets.

Uniffi cannot marshal `impl TryInto<ColorTarget>` across the FFI boundary, so
this mobile shim dispatches through `TargetHandle` — the same pattern used by
`Renderer::render_mobile`.

## Example

```rust
// hidden mobile binding; no public example
```
