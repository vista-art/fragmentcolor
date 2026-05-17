import org.fragmentcolor.*

val foliage = Material.pbr()?.alphaMode(AlphaMode.Mask).alphaCutoff(0.3)

val glass = Material.pbr()?.baseColor(listOf(0.9f, 0.95f, 1.0f, 0.25f)).alphaMode(AlphaMode.Blend)

val solid = Material.pbr()?.alphaMode(AlphaMode.Opaque)