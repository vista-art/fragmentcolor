import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(16, 16), TextureFormat.RGBA)
val id = texture.id()

renderer.unregisterTexture(id)