import FragmentColor

// Encoded path — single tuple, no extra method.
let chain = try TextureMipChain.prepare((encoded_png_bytes, TextureFormat.rgba8UnormSrgb))

// Raw pixel path — same method, just include the size in the tuple.
let chain_raw = try TextureMipChain.prepare((
    raw_rgba,
    TextureFormat.rgba8UnormSrgb,
    [8, 8],
))

// Hand the chain to the unified create_texture entry — same vocabulary.
let renderer = Renderer()
let texture = try await renderer.createTexture(chain)