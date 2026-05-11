from fragmentcolor import TextureFormat, Mipmap

pixels = [200] * (4 * 4 * 4)
chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [4, 4])
_ = chain.format()