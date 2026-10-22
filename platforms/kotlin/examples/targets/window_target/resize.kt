import org.fragmentcolor.*
val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))

val renderer = Renderer()
val target = renderer.createTarget(canvas)

target.resize(listOf(1024, 768))