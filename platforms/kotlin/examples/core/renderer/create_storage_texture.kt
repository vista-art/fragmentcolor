import org.fragmentcolor.*

val r = Renderer()
// Empty storage texture â same single create_storage_texture entry.
val tex = r.createStorageTexture((arrayOf(64, 64), TextureFormat.Rgba))

// Pre-seeded with bytes â same method, three-tuple form.
val pixels = Array(64 * 64 * 4) { 0 }
val tex2 = r.createStorageTexture((arrayOf(64, 64), TextureFormat.Rgba, pixels))