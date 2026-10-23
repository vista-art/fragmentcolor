import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64u, 64u), TextureFormat.RGBA)
val id = texture.id()