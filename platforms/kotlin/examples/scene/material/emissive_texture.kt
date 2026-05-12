import org.fragmentcolor.*

val renderer = Renderer()
val glow = renderer.createTexture(arrayOf(255, 0, 0, 255, 255,   0, 0, 255, 255,   0, 0, 255, 255,   0, 0, 255, await))
val mat = Material.pbr().emissive(listOf(0.8f, 0.0f, 0.0f)).emissiveTexture(glow)