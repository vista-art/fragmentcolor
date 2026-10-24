import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(16u, 16u), TextureFormat.RGBA)
val id = texture.id()

renderer.unregisterTexture(id)