import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([16, 16], TextureFormat.rgba))
let id = texture.id()

try renderer.unregisterTexture(id)