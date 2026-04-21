import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture([16, 16], TextureFormat.Rgba, null)
val id = *texture.id()

renderer.unregisterTexture(id)