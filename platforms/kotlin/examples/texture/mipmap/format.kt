import org.fragmentcolor.*

val pixels = ByteArray(4 * 4 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(4, 4))
val format = chain.format()