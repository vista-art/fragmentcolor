
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val shader = Shader.default()
val pass = Pass("blend with previous")
pass.addShader(shader)
pass.loadPrevious()

renderer.render(pass, target)