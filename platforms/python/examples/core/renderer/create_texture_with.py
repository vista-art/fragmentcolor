from fragmentcolor import Renderer
renderer = Renderer()
pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
]
tex = renderer.create_texture_with(pixels, [2, 2])