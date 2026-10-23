
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

val shader = Shader.default()
val pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport(ScreenRegion(listOf(0u, 0u), listOf(32u, 32u)))

renderer.render(pass, target)