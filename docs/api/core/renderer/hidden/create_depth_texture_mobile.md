# Renderer.createDepthTexture (mobile)

Mobile wrapper for `Renderer::create_depth_texture`. Takes the `Size` record, layers included,
because uniffi cannot marshal `impl Into<Size>`. The Swift and Kotlin extensions add the
`[width, height]` list overloads.

Hidden from public website; IDE hover via lsp_doc.

## Example

```swift
let depth = try await renderer.createDepthTexture([800, 600])
```

```kotlin
val depth = renderer.createDepthTexture(listOf(800u, 600u))
```
