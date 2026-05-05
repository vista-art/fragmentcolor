# Renderer::create_target_ios()

iOS-only constructor that wraps an existing `CAMetalLayer` into a
`WindowTarget`. The Swift extension file re-exposes this as
`Renderer.createTarget(layer:)` so the public API reads the same as every
other platform.

Caller passes the layer pointer as `u64`:

```swift
let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
let target = try renderer.createTargetIos(metalLayerPtr: ptr)
```

Exposed as synchronous because `wgpu::SurfaceTargetUnsafe` holds a raw
pointer and can't be held across an `await` in a `Send` future.

## Example

```rust
// hidden file; no public example (platform-specific)
```
