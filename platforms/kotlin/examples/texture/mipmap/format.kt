import org.fragmentcolor.*

val pixels = ByteArray(4 * 4 * 4)
val chain = Mipmap.build(pixels, TextureFormat.RGBA8_UNORM_SRGB, Size(width=4u, height=4u, depth=null))
