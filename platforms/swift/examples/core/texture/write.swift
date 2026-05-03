import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([64, 64], TextureFormat.rgba))
let frame_bytes = Array(repeating: 0, count: 64 * 64 * 4)

try texture.write(frame_bytes)