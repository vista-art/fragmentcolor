
from fragmentcolor import Renderer

renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255,255,255,255]
tex = renderer.create_texture_with_size(pixels, [1, 1])
a = tex.aspect()
