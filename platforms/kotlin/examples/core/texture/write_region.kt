import org.fragmentcolor.*
val renderer = Renderer()
val texture = renderer.createStorageTexture(Size(width=64u, height=32u, depth=null), TextureFormat.RGBA, null, null)
val bytes = ByteArray(64 * 32 * 4)

// Simple sub-rectangle update.
texture.writeRegion(bytes, TextureRegionMobile(0u, 0u, 0u, 64u, 32u, 0u, null, null))

// Explicit data layout (advanced â when source rows are padded).
val region = TextureRegionMobile(0u, 0u, 0u, 64u, 32u, 0u, 256u, 32u)
texture.writeRegion(bytes, region)