# Renderer.createTarget (Android)

Android-specific constructor that wraps a pre-acquired `ANativeWindow` pointer into a
`WindowTarget` on this `Renderer`. A Kotlin extension file re-exposes this as
`Renderer.createTarget(surface)` so the public API reads the same as every other platform.

The extension gets the pointer from the `Surface` through a JNI helper in
`libfragmentcolor.so`, passes it as `ULong`, and releases its own reference afterwards.
The target keeps a reference of its own.

Exposed as synchronous because `ANativeWindow*` holds a raw pointer that cannot be held
across `await` in a `Send` future.

## Example

```kotlin
// In RendererExtensions.kt:
fun Renderer.createTarget(surface: Surface): MobileWindowTarget {
    val window = SurfaceBridge.native_window_from_surface(surface)
    require(window != 0L) { "Failed to acquire an ANativeWindow from Surface" }
    try {
        return createTarget(window.toULong())
    } finally {
        SurfaceBridge.release_native_window(window)
    }
}
```
