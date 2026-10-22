import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = listOf(255, 255, 255, 255)

val texture = renderer.createTexture(pixels, listOf(1, 1))
val opts = SamplerOptions(repeatX = true, repeatY = true, smooth = true, compare = null)
texture.setSamplerOptions(opts)