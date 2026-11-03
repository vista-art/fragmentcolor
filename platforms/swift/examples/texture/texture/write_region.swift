import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 32], TextureFormat.rgba)
let bytes = Array(repeating: 0, count: 64 * 32 * 4)

// Simple sub-rectangle update.
try texture.writeRegion(bytes, [0, 0, 64, 32])

// Explicit data layout, for when the source rows are padded.
let region = try TextureRegion.from([0, 0, 64, 32]).withStride(256).withRows(32)
try texture.writeRegion(bytes, region)