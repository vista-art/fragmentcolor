from fragmentcolor import Renderer, Size, SamplerOptions
renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255,255,255,255]
tex = renderer.create_texture_with_size(pixels, Size.from((1,1)))
opts = SamplerOptions { repeat_x: true, repeat_y: true, smooth: true, compare: None }
tex.set_sampler_options(opts)