import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(16u, 16u))
val shader = Shader.default()

// A GPU error in this frame returns as a DeviceError.
renderer.render(shader, target)