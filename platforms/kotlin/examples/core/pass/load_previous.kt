
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([64, 64])

val shader = Shader.default()
val pass = Pass("blend with previous")
pass.addShader(shader)
pass.loadPrevious()

renderer.render(pass, target)