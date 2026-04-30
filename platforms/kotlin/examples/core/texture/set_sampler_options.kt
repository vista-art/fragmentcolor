import org.fragmentcolor.*

val renderer = Renderer()
val pixels: ByteArray = byteArrayOf(255.toByte(), 255.toByte(), 255.toByte(), 255.toByte())
val texture = renderer.createTextureWithSize(pixels, Size(1u, 1u))

val opts = SamplerOptions(repeatX = true, repeatY = true, smooth = true, compare = null)
texture.setSamplerOptions(opts)