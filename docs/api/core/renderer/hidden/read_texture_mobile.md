# Renderer::readTexture(textureId)

Mobile (Swift / Kotlin) wrapper for `Renderer::read_texture`. Uniffi exposes this as a Swift `async throws` function / Kotlin `suspend fun` automatically. Returns the tightly-packed pixel bytes for the registered texture in its native format.

## Example

```swift
let bytes = try await renderer.readTexture(texture.id())
```

```kotlin
val bytes = renderer.readTexture(texture.id())
```
