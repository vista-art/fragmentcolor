
import org.fragmentcolor.*

val renderer = Renderer()
// 1x1 RGBA (white) raw pixel bytes
val pixels = [255,255,255,255]
val tex = renderer.createTextureWithSize(pixels, [1, 1])
val a = tex.aspect()