import org.fragmentcolor.*

val renderer = Renderer()
val normal_map = renderer.createTexture(arrayOf(128, 128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, 128,   128, 255, 255, await))
val mat = Material.pbr(renderer).normalTexture(normal_map).normalScale(1.2)