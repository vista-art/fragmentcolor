import org.fragmentcolor.*

val material = Material.pbr()
material.shader().set("camera.viewProj", listOf(arrayOf(1.0f, 0.0f, 0.0f, 0.0)f, arrayOf(0.0f, 1.0f, 0.0f, 0.0)f, arrayOf(0.0f, 0.0f, 1.0f, 0.0)f, arrayOf(0.0f, 0.0f, 0.0f, 1.0)f, .0f))
material.shader().set("camera.position", floatArrayOf(0.0f, 0.0f, 5.0f))