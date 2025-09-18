renderer = Renderer()
size = Size.from((2, 2))
pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
]
tex = renderer
    .create_texture_with(pixels, size)
    