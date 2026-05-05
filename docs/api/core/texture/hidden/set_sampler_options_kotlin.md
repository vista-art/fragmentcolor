# Texture.setSamplerOptions(opts)

Kotlin wrapper for `Texture::set_sampler_options`. `SamplerOptions` and
`CompareFunction` are uniffi-exported records / enums.

## Example

```kotlin
import org.fragmentcolor.*

val renderer = Renderer()
val pixels: ByteArray = byteArrayOf(255.toByte(), 255.toByte(), 255.toByte(), 255.toByte())
val options = TextureOptions(
    size = Size(width = 1u, height = 1u, depth = null),
    format = TextureFormat.RGBA8_UNORM_SRGB,
    sampler = SamplerOptions(repeatX = false, repeatY = false, smooth = true, compare = null),
    mipmaps = false,
    usage = null,
)
val texture = renderer.createTexture(TextureInputMobile.Bytes(pixels), options)

val opts = SamplerOptions(repeatX = true, repeatY = true, smooth = true, compare = null)
texture.setSamplerOptions(opts)
```
