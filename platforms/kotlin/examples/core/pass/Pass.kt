
import org.fragmentcolor.*

val renderer = Renderer()
val canvas = document.createElement("canvas")
val target = renderer.createTarget(canvas)
val shader = Shader.default()

val pass = Pass("First Pass")
pass.addShader(shader)

val pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
renderer.render(pass, target)

// vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render(arrayOf(pass, pass2), target)