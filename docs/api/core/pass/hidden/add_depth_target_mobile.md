# Pass::addDepthTarget (mobile)

Mobile binding for `Pass::add_depth_target`. Accepts a `TargetHandle::Texture`
variant wrapping a depth-format texture (`Depth32Float` or similar).

Uniffi cannot marshal `impl TryInto<DepthTarget>` across the FFI boundary, so
this mobile shim dispatches through `TargetHandle` — the same pattern used by
`Renderer::render_mobile`.

## Example

```rust
// hidden mobile binding; no public example
```
