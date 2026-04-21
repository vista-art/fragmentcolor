
import org.fragmentcolor.*

val renderer = Renderer()

// Use your platform's windowing system to create a window.
val canvas = document.createElement('canvas')

val target = renderer.createTarget(canvas)

// To animate, render again in your event loop...
renderer.render(Shader(""), target)
renderer.render(Shader(""), target)