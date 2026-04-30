import org.fragmentcolor.*
val renderer = Renderer()
val pixels = [
    255,0,0,255,   0,255,0,255,
    0,0,255,255,   255,255,255,255,
]
val tex = renderer.createTextureWithSize(pixels, arrayOf(2, 2))