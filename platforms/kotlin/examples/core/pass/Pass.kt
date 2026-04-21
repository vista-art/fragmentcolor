
import org.fragmentcolor.*

val renderer = Renderer()
val canvas = document.createElement('canvas')
val target = renderer.createTarget(canvas)
val shader = Shader.default()

val pass = Pass("First Pass")
pass.addShader(shader)

val pass2 = Pass("Second Pass")
pass2.addShader(shader)

// standalone
renderer.render(pass, target)

// using a Frame
val frame = Frame()
frame.addPass(pass)
frame.addPass(pass2)
renderer.render(frame, target)

// vector of passes (consume them)
renderer.render([pass, pass2], target)