# Renderer.createDepthTexture (mobile)

Mobile wrapper for `Renderer::create_depth_texture`. Takes explicit `width` and `height` as
`UInt32` because uniffi cannot marshal `impl Into<Size>`.

Hidden from public website; IDE hover via lsp_doc.

## Example

```swift
let depth = try await renderer.createDepthTexture(width: 800, height: 600)
```

```kotlin
val depth = renderer.createDepthTexture(800u, 600u)
```
