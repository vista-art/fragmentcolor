import FragmentColor
let renderer = Renderer()
let texture = try await renderer.createStorageTexture([64, 64], TextureFormat.Rgba, nil)
let frame_bytes = Array(repeating: 0, count: 64 * 64 * 4)

texture.write(frame_bytes)