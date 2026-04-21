
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([64, 64])

val shader = Shader.default()
val pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport([(0, 0), (32, 32)])

renderer.render(pass, target)