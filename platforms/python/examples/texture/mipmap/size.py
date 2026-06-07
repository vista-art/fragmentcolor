from fragmentcolor import TextureFormat, Mipmap

pixels = [0] * (16 * 16 * 4)
chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16])
(width, height) = chain.size()
_ = (width, height)