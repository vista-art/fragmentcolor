import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64, 64), TextureFormat.RGBA)
val id = texture.id()