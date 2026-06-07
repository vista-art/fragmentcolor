import org.fragmentcolor.*

val foliage = Material.pbr().alphaMode(AlphaMode.MASK).alphaCutoff(0.3f)

val glass = Material.pbr().baseColor(listOf(0.9f, 0.95f, 1.0f, 0.25f)).alphaMode(AlphaMode.BLEND)

val solid = Material.pbr().alphaMode(AlphaMode.OPAQUE)