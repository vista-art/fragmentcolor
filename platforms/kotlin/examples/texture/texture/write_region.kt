import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(Size(width=64u, height=32u, depth=null), TextureFormat.RGBA, null, null)
val bytes = ByteArray(64 * 32 * 4)

// Simple sub-rectangle update.
texture.writeRegion(bytes, listOf(0, 0, 64, 32))

// Explicit data layout (advanced — when source rows are padded).
val region = TextureRegionMobile.from(listOf(0, 0, 64, 32)).withStride(256).withRows(32)
texture.writeRegion(bytes, region)