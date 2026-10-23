import org.fragmentcolor.*
val pixels = ByteArray(4 * 4 * 4) { 200.toByte() }

val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, listOf(4u, 4u))
val format = chain.format()