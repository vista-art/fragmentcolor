import org.fragmentcolor.*

val renderer = Renderer()
val normal_map_bytes = listOf(128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255)
val normal_map = renderer.createTexture(normal_map_bytes, listOf(2u, 2u))
val mat = Material.pbr().normalTexture(normal_map).normalScale(1.2f)