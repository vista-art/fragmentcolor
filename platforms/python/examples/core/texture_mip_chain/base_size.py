from fragmentcolor import TextureFormat, TextureMipChain

pixels = [0] * (16 * 16 * 4)
chain = TextureMipChain.prepare((
    pixels.as_slice(),
    TextureFormat.Rgba8UnormSrgb,
    [16, 16],
))
(width, height) = chain.base_size()
_ = (width, height)