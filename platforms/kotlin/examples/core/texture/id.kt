import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, null, null)
val id = texture.id()