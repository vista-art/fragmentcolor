import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val ao_bytes = listOf(220, 0, 0, 255, 180, 0, 0, 255, 200, 0, 0, 255, 160, 0, 0, 255)
val mat = Material.pbr().occlusionTexture(ao_bytes, listOf(2u, 2u))