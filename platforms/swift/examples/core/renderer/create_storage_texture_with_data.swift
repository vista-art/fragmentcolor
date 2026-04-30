import FragmentColor

let r = Renderer()
let seed = Array(repeating: 0, count: 8 * 8 * 4)
let tex = try await r.createStorageTextureWithData([8, 8], TextureFormat.Rgba, seed, nil)