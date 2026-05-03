
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val shader = Shader.default()
val pass = Pass("solid background")
pass.addShader(shader)

pass.setClearColor(listOf(0.1f, 0.2f, 0.3f, 1.0f))

renderer.render(pass, target)