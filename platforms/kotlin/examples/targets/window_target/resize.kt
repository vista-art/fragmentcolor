import org.fragmentcolor.*
val surfaceTexture = android.graphics.SurfaceTexture(0)
val canvas = android.view.Surface(surfaceTexture)

val renderer = Renderer()
val target = renderer.createTarget(canvas)

target.resize(listOf(1024u, 768u))
surfaceTexture.release()