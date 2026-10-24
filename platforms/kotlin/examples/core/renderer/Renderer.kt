
import org.fragmentcolor.*
val surfaceTexture = android.graphics.SurfaceTexture(0)
val canvas = android.view.Surface(surfaceTexture)

val renderer = Renderer()

// The canvas comes from your platform's windowing system
val target = renderer.createTarget(canvas)
val texture_target = renderer.createTextureTarget(listOf(16u, 16u))

// RENDERING
renderer.render(Shader.default(), texture_target)

// That's it. Welcome to FragmentColor!
surfaceTexture.release()