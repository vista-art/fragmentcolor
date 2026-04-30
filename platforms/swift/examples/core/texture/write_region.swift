import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 32], TextureFormat.Rgba, nil)
let bytes = Array(repeating: 0, count: 64 * 32 * 4)

// Simple sub-rectangle update.
texture.writeRegion(bytes, [0, 0, 64, 32])

// Explicit data layout (advanced â when source rows are padded).
let region = TextureRegion.from([0, 0, 64, 32]).withStride(256).withRows(32)
texture.writeRegion(bytes, region)