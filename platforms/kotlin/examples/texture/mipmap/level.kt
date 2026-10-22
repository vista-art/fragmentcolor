import org.fragmentcolor.*
val pixels = ByteArray(8 * 8 * 4)

val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(8, 8))
val level_zero_bytes = chain.level(0u)