import org.fragmentcolor.*

val renderer = Renderer()
val mr_map_bytes = listOf(0, 200, 50, 255, 0, 240, 80, 255, 0, 180, 30, 255, 0, 220, 60, 255)
val mr_map = renderer.createTexture(mr_map_bytes, listOf(2, 2))
val mat = Material.pbr().metallicRoughnessTexture(mr_map)