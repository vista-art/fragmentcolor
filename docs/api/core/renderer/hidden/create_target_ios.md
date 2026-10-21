# Renderer::create_target_ios()

iOS-only constructor that wraps an existing `CAMetalLayer` into a
`WindowTarget`. Swift sees it as `createTarget(metalLayerPtr:)`. The Swift
extension file re-exposes it as `Renderer.createTarget(layer)` so the public
API reads the same as every other platform.

The extension passes the layer pointer as `UInt64`:

```swift
let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
let target = try renderer.createTarget(metalLayerPtr: ptr)
```

Exposed as synchronous because `wgpu::SurfaceTargetUnsafe` holds a raw
pointer and can't be held across an `await` in a `Send` future.

## Example

```rust
// hidden file; no public example (platform-specific)
```
