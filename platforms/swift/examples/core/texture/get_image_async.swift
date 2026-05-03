import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([64, 64], TextureFormat.rgba))
try texture.write(Array(repeating: UInt8(0), count: 64 * 64 * 4))

let bytes = try await texture.getImage()
