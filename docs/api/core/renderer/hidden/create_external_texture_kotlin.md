# Renderer.createExternalTexture(sourcePtr)

Kotlin wrapper for `Renderer::create_external_texture`. Takes a raw
`ULong` pointer to a `SurfaceTexture` (or any equivalent native handle)
so uniffi can marshal it across the FFI boundary.

Currently a stub — every call returns
`FragmentColorError.Render("not implemented yet")` until the Android
plumbing to map a `SurfaceTexture` into a `wgpu::ExternalTexture` lands.

## Example

```kotlin
import org.fragmentcolor.*
// Once supported:
//   val renderer = Renderer()
//   val surfaceTexture: SurfaceTexture = /* from MediaCodec / Camera2 */
//   val ptr: ULong = surfaceTexture.nativeHandle()  // hypothetical helper
//   val handle = renderer.createExternalTexture(ptr)
```
