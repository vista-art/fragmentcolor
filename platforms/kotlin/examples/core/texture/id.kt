import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture((arrayOf(64, 64), TextureFormat.Rgba))
val id = texture.id()