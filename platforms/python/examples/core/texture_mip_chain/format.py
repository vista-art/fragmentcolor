from fragmentcolor import TextureFormat, TextureMipChain

pixels = [200] * (4 * 4 * 4)
chain = TextureMipChain.prepare((
    pixels.as_slice(),
    TextureFormat.Rgba8UnormSrgb,
    [4, 4],
))
_ = chain.format()