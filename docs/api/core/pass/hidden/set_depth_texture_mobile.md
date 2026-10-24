# Pass::setDepthTexture (mobile)

Mobile binding for `Pass::set_depth_target` with a depth `Texture` from
`Renderer::create_depth_texture`. Uniffi cannot marshal `impl TryInto<DepthTarget>`
and has no method overloading, so the `Texture` form gets its own entry point.
The Swift and Kotlin extensions expose it as `setDepthTarget(texture)`.

## Example

```rust
// hidden mobile binding; no public example
```
