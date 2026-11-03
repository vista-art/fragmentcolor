from fragmentcolor import Mipmap, TextureFormat
pixels = bytes(16 * 16 * 4)

chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, size=[16, 16])
size = chain.size()
width = size.width
height = size.height