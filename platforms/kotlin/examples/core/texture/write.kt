import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null)
val frame_bytes = [0u8; 64 * 64 * 4]

texture.write(frame_bytes)