import org.fragmentcolor.*
val pixels = ByteArray(8 * 8 * 4)

val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(8u, 8u))
val level_zero_bytes = chain.levels()[0]