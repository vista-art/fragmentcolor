import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(listOf(64, 32), TextureFormat.RGBA)
val bytes = ByteArray(64 * 32 * 4)

// Simple sub-rectangle update.
texture.writeRegion(bytes, listOf(0, 0, 64, 32))

// Explicit data layout, for when the source rows are padded.
val region = TextureRegion.from(listOf(0, 0, 64, 32)).withStride(256).withRows(32)
texture.writeRegion(bytes, region)