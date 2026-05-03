
import org.fragmentcolor.*

val renderer = Renderer()

// Use your platform's windowing system to create a window
// HEADLESS: canvas creation not needed on Android

// Create a Target from it
val target = renderer.createTextureTarget(800u, 600u)
val texture_target = renderer.createTextureTarget(16u, 16u)

// RENDERING
renderer.render(Shader(""), texture_target)

// That's it. Welcome to FragmentColor!