import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64u, 32u), TextureFormat.RGBA)
val bytes = ByteArray(64 * 32 * 4)

// Simple sub-rectangle update.
texture.writeRegion(bytes, listOf(0u, 0u, 64u, 32u))

// Explicit data layout, for when the source rows are padded.
val region = TextureRegion.from(listOf(0u, 0u, 64u, 32u)).withStride(256u).withRows(32u)
texture.writeRegion(bytes, region)