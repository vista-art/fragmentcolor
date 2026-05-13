import org.fragmentcolor.*

val renderer = Renderer()
val material = Material.pbr(renderer)
val sun = Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f))
sun.bind(material.shader())