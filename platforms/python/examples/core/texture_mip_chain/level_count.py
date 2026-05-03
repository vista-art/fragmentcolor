from fragmentcolor import TextureFormat, TextureMipChain

pixels = [0] * (8 * 8 * 4)
chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8])
count = chain.level_count()
_ = count
