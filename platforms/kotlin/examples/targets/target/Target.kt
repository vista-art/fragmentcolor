
import org.fragmentcolor.*
val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))

val renderer = Renderer()

// The canvas comes from your platform's windowing system.
val target = renderer.createTarget(canvas)

// To animate, render again in your event loop...
renderer.render(Shader.default(), target)
renderer.render(Shader.default(), target)