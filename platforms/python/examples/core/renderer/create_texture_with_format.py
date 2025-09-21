from fragmentcolor import Renderer, Size, TextureFormat
renderer = Renderer()
bytes = std.fs.read("logo.png")
tex = renderer.create_texture_with_format(bytes, TextureFormat.Rgba)