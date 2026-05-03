
import org.fragmentcolor.*

// Use your platform's windowing system to create a window.
// HEADLESS: canvas creation not needed on Android

val renderer = Renderer()
val target = renderer.createTextureTarget(800u, 600u)

renderer.render(Shader(""), target)