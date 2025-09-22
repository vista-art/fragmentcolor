from fragmentcolor import Renderer
renderer = Renderer()
pixels = [255,255,255,255]
tex = renderer.create_texture_with_size(pixels, [1,1])
sz = tex.size