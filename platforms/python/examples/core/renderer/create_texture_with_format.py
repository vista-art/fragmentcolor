from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
image = open("logo.png", "rb").read()
tex = renderer.create_texture_with_format(image, TextureFormat.Rgba)