import org.fragmentcolor.*
val renderer = Renderer()
val pixels = arrayOf(255,255,255,255)
val tex = renderer.createTextureWithSize(pixels, arrayOf(1,1))
val sz = tex.size()