
import org.fragmentcolor.*

val renderer = Renderer()

// Use your platform's windowing system to create a window
val canvas = document.createElement('canvas')

// Create a Target from it
val target = renderer.createTarget(canvas)
val texture_target = renderer.createTextureTarget([16, 16])

// RENDERING
renderer.render(Shader(""), texture_target)

// That's it. Welcome to FragmentColor!