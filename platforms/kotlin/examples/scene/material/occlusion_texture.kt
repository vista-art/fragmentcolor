import org.fragmentcolor.*

val renderer = Renderer()
val ao_bytes = listOf(220, 0, 0, 255, 180, 0, 0, 255, 200, 0, 0, 255, 160, 0, 0, 255)
val ao = renderer.createTexture(ao_bytes, listOf(2, 2))
val mat = Material.pbr().occlusionTexture(ao)