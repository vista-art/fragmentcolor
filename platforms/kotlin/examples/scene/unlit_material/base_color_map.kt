import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val pixels = listOf(255, 128, 0, 255)
val material = UnlitMaterial()
val map = material.baseColorMap().texture(pixels, listOf(1u, 1u))