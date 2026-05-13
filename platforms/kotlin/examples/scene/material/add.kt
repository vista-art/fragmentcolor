import org.fragmentcolor.*

val renderer = Renderer()
val material = Material.pbr(renderer)

val camera = Camera.perspective(60.0.toRadians(), 16.0 / 9.0, 0.1, 100.0).lookAt(listOf(0.0f, 1.0f, 5.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
val sun = Light.directional(listOf(0.3f, -1.0f, -0.4f), listOf(1.0f, 0.95f, 0.9f))

material.add(camera).add(sun)

// Updating the camera later is enough — the Material picks the new
// view_proj up at the next render without re-adding.
camera.lookAt(listOf(3.0f, 1.0f, 5.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))