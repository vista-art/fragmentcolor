
import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = arrayOf(255,255,255,255)
val tex = renderer.createTextureWithSize(pixels, arrayOf(1, 1))
val a = tex.aspect()