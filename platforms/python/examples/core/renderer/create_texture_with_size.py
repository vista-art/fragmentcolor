from fragmentcolor import Renderer, Size
renderer = Renderer()
pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
]
tex = renderer.create_texture_with_size(pixels, [2, 2])