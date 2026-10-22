# Renderer.unregisterTexture (mobile)

Mobile wrapper for `Renderer::unregister_texture`. The raw uniffi method takes
the texture id as a `UInt64`. The Swift and Kotlin extensions take the
`TextureId` that `texture.id()` returns, as every other binding does.

Hidden from public website; IDE hover via lsp_doc.

## Example

```swift
try renderer.unregisterTexture(texture.id())
```

```kotlin
renderer.unregisterTexture(texture.id())
```
