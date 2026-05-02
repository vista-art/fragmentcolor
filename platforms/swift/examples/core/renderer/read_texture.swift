import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([64, 64], TextureFormat.Rgba))
texture.write(Array(repeating: 0, count: 64 * 64 * 4))

let bytes = renderer.readTexture(texture.id())