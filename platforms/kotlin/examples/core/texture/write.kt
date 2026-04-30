import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(arrayOf(64, 64), TextureFormat.Rgba, null)
val frame_bytes = Array(64 * 64 * 4) { 0 }

texture.write(frame_bytes)