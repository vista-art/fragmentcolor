import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64, 64), TextureFormat.RGBA)
texture.write(ByteArray(64 * 64 * 4))

val bytes = renderer.readTexture(texture.id())