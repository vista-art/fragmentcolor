import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(Size(width=16u, height=16u, depth=null), TextureFormat.RGBA, null, null)
val id = texture.id()

renderer.unregisterTexture(id)