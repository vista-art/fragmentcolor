import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null)
val id = *texture.id()
val frame = [0u8; 64 * 64 * 4]

renderer.updateTexture(id, frame)