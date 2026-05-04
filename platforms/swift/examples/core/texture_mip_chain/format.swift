import FragmentColor

let pixels = Array(repeating: 200, count: 4 * 4 * 4)
let chain = try TextureMipChain.prepare((
    pixels,
    TextureFormat.rgba8UnormSrgb,
    [4, 4],
))
let _ = chain.format()