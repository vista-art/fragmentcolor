import FragmentColor

let r = Renderer()
// Empty storage texture — same single create_storage_texture entry.
let tex = try await r.createStorageTexture(([64, 64], TextureFormat.rgba))

// Pre-seeded with bytes — same method, three-tuple form.
let pixels = Array(repeating: 0, count: 64 * 64 * 4)
let tex2 = try await r.createStorageTexture(([64, 64], TextureFormat.rgba, pixels))