import org.fragmentcolor.*

// Deep blue shadows, a warm middle tone and full light, one texel each.
val ramp = listOf(20, 30, 90, 255, 20, 30, 90, 255, 200, 140, 110, 255, 255, 255, 255, 255)
val material = ToonMaterial()
val map = material.rampMap().texture(ramp, listOf(4u, 1u))