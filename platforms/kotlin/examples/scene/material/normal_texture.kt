import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val normal_map_bytes = listOf(128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 255, 255)
val mat = Material.pbr().normalTexture(normal_map_bytes, listOf(2u, 2u)).normalScale(1.2f)