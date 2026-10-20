import org.fragmentcolor.*

val pixels = ByteArray(16 * 16 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, Size(width=16u, height=16u, depth=null))
val (width, height) = chain.size()