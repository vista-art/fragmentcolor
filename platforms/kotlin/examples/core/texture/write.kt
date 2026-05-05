import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, null, null)
val frame_bytes = ByteArray(64 * 64 * 4)

texture.write(frame_bytes)