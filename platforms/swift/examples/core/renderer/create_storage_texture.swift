import FragmentColor

let r = Renderer()

// Empty storage texture.
let tex = try await r.createStorageTexture(([64, 64], TextureFormat.rgba))

// Pre-seeded with bytes.
let pixels = Array(repeating: 0, count: 64 * 64 * 4)
let tex2 = try await r.createStorageTexture(([64, 64], TextureFormat.rgba, pixels))