import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([64, 32], TextureFormat.rgba))
let bytes = Array(repeating: 0, count: 64 * 32 * 4)

// Simple sub-rectangle update.
try texture.writeRegion(bytes, [0, 0, 64, 32])

// Explicit data layout (advanced — when source rows are padded).
let region = TextureRegionMobile.from([0, 0, 64, 32]).withStride(256).withRows(32)
try texture.writeRegion(bytes, region)