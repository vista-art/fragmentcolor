import org.fragmentcolor.*

val renderer = Renderer()
val ao = renderer.createTexture(arrayOf(220, 0, 0, 255, 180,   0, 0, 255, 200,   0, 0, 255, 160,   0, 0, 255, await))
val mat = Material.pbr().occlusionTexture(ao)