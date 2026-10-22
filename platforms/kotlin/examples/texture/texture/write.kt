import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64, 64), TextureFormat.RGBA)
val frame_bytes = ByteArray(64 * 64 * 4)

texture.write(frame_bytes)