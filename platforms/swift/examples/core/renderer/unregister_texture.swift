import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture(([16, 16], TextureFormat.Rgba))
let id = texture.id()

renderer.unregisterTexture(id)