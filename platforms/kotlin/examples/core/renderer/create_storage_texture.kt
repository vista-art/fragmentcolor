import org.fragmentcolor.*

val r = Renderer()
// Empty storage texture â same single create_storage_texture entry.
val tex = r.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, null, null)

// Pre-seeded with bytes â same method, three-tuple form.
val pixels = ByteArray(64 * 64 * 4)
val tex2 = r.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, pixels, null)