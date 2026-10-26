# Renderer::create_target_ios()

iOS-only constructor that wraps an existing `CAMetalLayer` into a
`WindowTarget`. The Swift extension file exposes it as
`Renderer.createTarget(layer)`, so the public API reads the same as every
other platform.

The extension passes the layer's address. The constructor checks that the
address names a `CAMetalLayer` before it reads the layer's drawable size, and
throws `FragmentColorError` for any other object, for a null address, and for a
drawable size that is negative or NaN.

Exposed as synchronous because `wgpu::SurfaceTargetUnsafe` holds a raw
pointer and can't be held across an `await` in a `Send` future.

## Example

```rust
// hidden file; no public example (platform-specific)
```
