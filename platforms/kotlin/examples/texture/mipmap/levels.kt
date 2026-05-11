import org.fragmentcolor.*

val pixels = ByteArray(8 * 8 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, Size(width=8u, height=8u, depth=null))
val level_zero_bytes = chain.level(0u)
