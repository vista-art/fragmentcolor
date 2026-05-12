import org.fragmentcolor.*

val renderer = Renderer()
val texture = renderer.createTexture(arrayOf(255, 200, 120, 255, 255,  240, 180, 255, 230,  180, 100, 255, 255,  220, 150, 255, await))
val mat = Material.pbr().baseColorTexture(texture)