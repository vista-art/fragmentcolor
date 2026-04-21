import org.fragmentcolor.*
val renderer = Renderer()
val pixels = [255,255,255,255]
val tex = renderer.createTextureWithSize(pixels, [1,1])
val sz = tex.size()