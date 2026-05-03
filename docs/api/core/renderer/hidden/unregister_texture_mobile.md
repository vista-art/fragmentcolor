# Renderer.unregisterTexture (mobile)

Mobile wrapper for `Renderer::unregister_texture`. Accepts the raw `UInt64` texture ID
rather than a `TextureId` object, avoiding an extra uniffi::Object binding until the
texture agent lands the canonical TextureId binding.

Hidden from public website; IDE hover via lsp_doc.

## Example

```swift
let id = texture.id()
renderer.unregisterTexture(textureId: id)
```

```kotlin
val id = texture.id()
renderer.unregisterTexture(id)
```
