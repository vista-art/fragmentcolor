
import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = listOf(255, 255, 255, 255)
val tex = renderer.createTexture(pixels, listOf(1, 1))
val a = tex.aspect()