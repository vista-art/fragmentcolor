import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture((arrayOf(64, 64), TextureFormat.Rgba))
texture.write(Array(64 * 64 * 4) { 0 })

val bytes = texture.getImage()