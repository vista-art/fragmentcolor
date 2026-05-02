import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture((arrayOf(16, 16), TextureFormat.Rgba))
val id = texture.id()

renderer.unregisterTexture(id)