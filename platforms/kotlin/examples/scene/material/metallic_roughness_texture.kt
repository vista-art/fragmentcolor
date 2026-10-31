import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val mr_map_bytes = listOf(0, 200, 50, 255, 0, 240, 80, 255, 0, 180, 30, 255, 0, 220, 60, 255)
val mat = Material.pbr().metallicRoughnessTexture(mr_map_bytes, listOf(2u, 2u))