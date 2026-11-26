import org.fragmentcolor.*

val foliage = PbrMaterial().alphaMode(AlphaMode.MASK).alphaCutoff(0.3f)

val glass = PbrMaterial().baseColor(listOf(0.9f, 0.95f, 1.0f, 0.25f)).alphaMode(AlphaMode.BLEND)

val solid = PbrMaterial().alphaMode(AlphaMode.OPAQUE)