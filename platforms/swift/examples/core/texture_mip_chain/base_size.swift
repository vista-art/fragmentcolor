import FragmentColor

let pixels = Array(repeating: 0, count: 16 * 16 * 4)
let chain = try TextureMipChain.prepare((
    pixels,
    TextureFormat.rgba8UnormSrgb,
    [16, 16],
))
let size = chain.baseSize()
let _ = size