import org.fragmentcolor.*

val r = Renderer()
val seed = Array(8 * 8 * 4) { 0 }
val tex = r.createStorageTextureWithData(arrayOf(8, 8), TextureFormat.Rgba, seed, null)