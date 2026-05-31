# Renderer.createExternalTexture(sourcePtr)

Swift wrapper for `Renderer::create_external_texture`. Takes a raw `UInt64`
pointer to a `CVPixelBuffer` so uniffi can marshal it across the FFI
boundary; the public Swift extension wraps a real `CVPixelBuffer` and hands
the underlying pointer in.

Currently a stub: every call returns
`FragmentColorError.render("not implemented yet")` until the iOS plumbing
to map a `CVPixelBuffer` into a `wgpu::ExternalTexture` lands.

## Example

```swift
import FragmentColor
// Once supported:
//   let renderer = Renderer()
//   let pixelBuffer: CVPixelBuffer = /* from AVPlayerItemVideoOutput */
//   let ptr = UInt64(UInt(bitPattern: Unmanaged.passUnretained(pixelBuffer).toOpaque()))
//   let handle = try renderer.createExternalTexture(sourcePtr: ptr)
```
