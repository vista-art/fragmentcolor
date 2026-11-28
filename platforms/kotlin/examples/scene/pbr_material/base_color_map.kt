import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val tint = listOf(255, 128, 0, 255)
val material = PbrMaterial()
val map = material.baseColorMap().texture(tint, listOf(1u, 1u))