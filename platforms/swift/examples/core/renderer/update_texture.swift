import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, nil)
let id = *texture.id()
let frame = [0u8; 64 * 64 * 4]

renderer.updateTexture(id, frame)