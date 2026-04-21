import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([64, 64], TextureFormat.Rgba, null)
val id = *texture.id()