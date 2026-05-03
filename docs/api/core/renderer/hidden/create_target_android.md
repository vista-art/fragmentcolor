# Renderer.createTarget (Android)

Android-specific constructor that wraps a pre-acquired `ANativeWindow` pointer into a
`WindowTarget`. A Kotlin extension file re-exposes this as `Renderer.createTarget(surface)`
so the public API reads the same as every other platform.

The Kotlin side obtains the pointer via `android.view.Surface.acquireNativeHandle()` or
by calling into the NDK directly, then passes it as `Long` / `ULong`.

Exposed as synchronous because `ANativeWindow*` holds a raw pointer that cannot be held
across `await` in a `Send` future.

## Example

```kotlin
// In RendererExtensions.kt:
fun Renderer.createTarget(surface: Surface): WindowTarget {
    val ptr = surface.acquireNativeHandle()
    return createTarget(nativeWindowPtr = ptr.toULong())
}
```
