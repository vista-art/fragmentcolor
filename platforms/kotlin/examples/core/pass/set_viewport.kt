
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64, 64))

val shader = Shader.default()
val pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport(ScreenRegion(listOf(0, 0), listOf(32, 32)))

renderer.render(pass, target)