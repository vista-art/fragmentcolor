import org.fragmentcolor.*

val renderer = Renderer()
val bronze = Material.pbr(renderer).baseColor(listOf(0.8f, 0.5f, 0.2f, 1.0f)).metallic(1.0).roughness(0.3)