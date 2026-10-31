import org.fragmentcolor.*
val surfaceTexture = android.graphics.SurfaceTexture(0)
val canvas = android.view.Surface(surfaceTexture)

val renderer = Renderer()
val target = renderer.createTarget(canvas)
val shader = Shader.default()
renderer.render(shader, target)

// The same frame, drawn where it can be read back.
val offscreen = renderer.createTextureTarget(listOf(800u, 600u))
renderer.render(shader, offscreen)
val image = offscreen.getImage()
surfaceTexture.release()