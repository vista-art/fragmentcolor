import FragmentColor

let pixels = Array(repeating: 0, count: 8 * 8 * 4)
let chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.Rgba8UnormSrgb,
    [8, 8],
))
let count = chain.levelCount()
let _ = count