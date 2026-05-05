import org.fragmentcolor.*

val r = Renderer()

// Empty storage texture.
val tex = r.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, null, null)

// Pre-seeded with bytes.
val pixels = ByteArray(64 * 64 * 4)
val tex2 = r.createStorageTexture(Size(width=64u, height=64u, depth=null), TextureFormat.RGBA, pixels, null)