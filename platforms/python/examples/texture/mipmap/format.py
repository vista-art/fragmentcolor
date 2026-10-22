from fragmentcolor import Mipmap, TextureFormat
pixels = bytes([200] * (4 * 4 * 4))

chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, size=[4, 4])
format = chain.format()