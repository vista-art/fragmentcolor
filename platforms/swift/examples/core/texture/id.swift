import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, nil)
let id = *texture.id()