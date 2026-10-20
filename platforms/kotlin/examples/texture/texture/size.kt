import org.fragmentcolor.*
val renderer = Renderer()
val pixels = listOf(255, 255, 255, 255)
val tex = renderer.createTexture(pixels, listOf(1, 1))
val sz = tex.size()
val width = sz.width
val height = sz.height