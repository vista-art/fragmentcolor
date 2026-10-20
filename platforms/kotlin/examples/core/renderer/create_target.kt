
import org.fragmentcolor.*
val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))

val renderer = Renderer()

// The canvas comes from your platform's windowing system.
val target = renderer.createTarget(canvas)