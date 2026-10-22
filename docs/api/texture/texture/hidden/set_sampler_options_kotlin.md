# Texture::set_sampler_options (Kotlin)

Kotlin override for `Texture::set_sampler_options`. The generic conversion
can't turn Rust's `SamplerOptions { .. }` struct literal into Kotlin, so this
page builds the options with the record's constructor.

## Example

```kotlin
import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = listOf(255, 255, 255, 255)

val texture = renderer.createTexture(pixels, listOf(1, 1))
val opts = SamplerOptions(repeatX = true, repeatY = true, smooth = true, compare = null)
texture.setSamplerOptions(opts)
```
