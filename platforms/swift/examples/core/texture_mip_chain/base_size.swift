import FragmentColor

let pixels = Array(repeating: 0, count: 16 * 16 * 4)
let chain = TextureMipChain.prepare((
    pixels.asSlice(),
    TextureFormat.rgba8UnormSrgb,
    [16, 16],
))
let (width, height) = chain.baseSize()
let _ = (width, height)