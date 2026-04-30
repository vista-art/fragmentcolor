import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(arrayOf(64, 32), TextureFormat.Rgba, null)
val bytes = Array(64 * 32 * 4) { 0 }

// Simple sub-rectangle update.
texture.writeRegion(bytes, arrayOf(0, 0, 64, 32))

// Explicit data layout (advanced â when source rows are padded).
val region = TextureRegion.from(arrayOf(0, 0, 64, 32)).withStride(256).withRows(32)
texture.writeRegion(bytes, region)