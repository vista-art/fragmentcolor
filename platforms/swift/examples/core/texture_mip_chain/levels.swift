import FragmentColor

let pixels = Array(repeating: 0, count: 8 * 8 * 4)
let chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.rgba8UnormSrgb,
    [8, 8],
))
let level_zero_bytes = chain.levels()[0]
let _ = level_zero_bytes