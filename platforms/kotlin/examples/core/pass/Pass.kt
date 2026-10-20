
import org.fragmentcolor.*
val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))

val renderer = Renderer()
val target = renderer.createTarget(canvas)
val shader = Shader.default()

val pass = Pass("First Pass")
pass.addShader(shader)

val pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
renderer.render(pass, target)

// vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render(listOf(pass, pass2), target)