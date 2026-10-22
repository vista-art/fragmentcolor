import org.fragmentcolor.*

val pixels = ByteArray(16 * 16 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(16, 16))
val (width, height) = chain.size()