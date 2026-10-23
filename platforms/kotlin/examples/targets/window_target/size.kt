import org.fragmentcolor.*
val surfaceTexture = android.graphics.SurfaceTexture(0)
val canvas = android.view.Surface(surfaceTexture)

val renderer = Renderer()
val target = renderer.createTarget(canvas)
val size = target.size()
surfaceTexture.release()