from fragmentcolor import Renderer
renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255,255,255,255]

texture = renderer.create_texture_with_size(pixels, [1,1])
opts = {"repeat_x": True, "repeat_y": True, "smooth": True, "compare": None}
texture.set_sampler_options(opts)

