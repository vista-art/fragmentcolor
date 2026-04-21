import FragmentColor
let renderer = Renderer()
let pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
]
let tex = try await renderer.createTextureWithSize(pixels, [2, 2])