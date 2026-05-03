import FragmentColor

// Encoded path â single tuple, no extra method.
let chain = TextureMipChain.prepare((encoded_png_bytes, TextureFormat.rgba8UnormSrgb))

// Raw pixel path â same method, just include the size in the tuple.
let chain_raw = TextureMipChain.prepare((
    raw_rgba.asSlice(),
    TextureFormat.rgba8UnormSrgb,
    [8, 8],
))

// Hand the chain to the unified create_texture entry â same vocabulary.
let renderer = Renderer()
let texture = try await renderer.createTexture(chain)