import org.fragmentcolor.*
val pixels = ByteArray(16 * 16 * 4)

val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(16u, 16u))
val size = chain.size()
val width = size.width
val height = size.height