import org.fragmentcolor.*

val pixels = ByteArray(16 * 16 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, Size(width=16u, height=16u, depth=null))
val tmp_size = chain.size()
val width = tmp_size.width
val height = tmp_size.height