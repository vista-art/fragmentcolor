import org.fragmentcolor.*

val renderer = Renderer()
val glow_bytes = listOf(255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255)
val glow = renderer.createTexture(glow_bytes, listOf(2, 2))
val mat = Material.pbr().emissive(listOf(0.8f, 0.0f, 0.0f)).emissiveTexture(glow)