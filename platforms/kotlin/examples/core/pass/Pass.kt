
import org.fragmentcolor.*

val renderer = Renderer()
// HEADLESS: canvas creation not needed on Android
val target = renderer.createTextureTarget(800u, 600u)
val shader = Shader.default()

val pass = Pass("First Pass")
pass.addShader(shader)

val pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
renderer.render(pass, target)

// vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render(listOf(pass, pass2), target)