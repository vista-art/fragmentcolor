from fragmentcolor import Mipmap, TextureFormat
pixels = bytes(8 * 8 * 4)

chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, size=[8, 8])
level_zero_bytes = chain.levels()[0]