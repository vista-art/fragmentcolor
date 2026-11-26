import org.fragmentcolor.*

// Raw pixels with their size upload when the scene loads.
val glow_bytes = listOf(255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255)
val material = PbrMaterial().emissive(listOf(0.8f, 0.0f, 0.0f))
val map = material.emissiveMap().texture(glow_bytes, listOf(2u, 2u))