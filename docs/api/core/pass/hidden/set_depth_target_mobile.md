# Pass::setDepthTarget (mobile)

Mobile binding for `Pass::set_depth_target` that takes a `TargetHandle`.

Uniffi cannot marshal `impl TryInto<DepthTarget>` across the FFI boundary, so
this mobile shim dispatches through `TargetHandle`, the same pattern used by
`Renderer::render_mobile`. A `TextureTarget` always has a color format, so this
form returns an error. Pass a depth `Texture` instead: the Swift and Kotlin
`setDepthTarget(texture)` extensions route it through `setDepthTexture`.

## Example

```rust
// hidden mobile binding; no public example
```
