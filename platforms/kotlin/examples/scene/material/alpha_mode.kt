import org.fragmentcolor.*

val renderer = Renderer()
val foliage = Material.pbr(renderer).alphaMode(AlphaMode.Mask).alphaCutoff(0.3)

val glass = Material.pbr(renderer).baseColor(listOf(0.9f, 0.95f, 1.0f, 0.25f)).alphaMode(AlphaMode.Blend)

val solid = Material.pbr(renderer).alphaMode(AlphaMode.Opaque)