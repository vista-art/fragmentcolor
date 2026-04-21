
import org.fragmentcolor.*

// Use your platform's windowing system to create a window.
val canvas = document.createElement('canvas')

val renderer = Renderer()
val target = renderer.createTarget(canvas)

renderer.render(Shader(""), target)