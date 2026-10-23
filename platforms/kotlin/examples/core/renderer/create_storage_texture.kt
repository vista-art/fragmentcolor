import org.fragmentcolor.*

val r = Renderer()

// Empty storage texture.
val tex = r.createStorageTexture(listOf(64u, 64u), TextureFormat.RGBA)

// Pre-seeded with bytes.
val pixels = ByteArray(64 * 64 * 4)
val tex2 = r.createStorageTexture(listOf(64u, 64u), TextureFormat.RGBA, pixels)