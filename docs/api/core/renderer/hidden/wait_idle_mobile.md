# Renderer::wait_idle()

Mobile wrapper for `Renderer::wait_idle`. Blocks until all GPU submissions on this
device have finished. Useful before readbacks to ensure deterministic ordering.

## Example

```swift
renderer.render(shader, target: target)
try renderer.waitIdle()
let bytes = target.getImage()
```

```kotlin
renderer.render(shader, target)
renderer.waitIdle()
val bytes = target.getImage()
```
