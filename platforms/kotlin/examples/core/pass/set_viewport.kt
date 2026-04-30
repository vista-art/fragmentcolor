
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(arrayOf(64, 64))

val shader = Shader.default()
val pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport(arrayOf((0, 0), (32, 32)))

renderer.render(pass, target)