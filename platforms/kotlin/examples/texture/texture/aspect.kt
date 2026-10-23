
import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = listOf(255, 255, 255, 255)
val tex = renderer.createTexture(pixels, listOf(1u, 1u))
val a = tex.aspect()