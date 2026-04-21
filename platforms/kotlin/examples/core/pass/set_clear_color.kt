
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([64, 64])

val shader = Shader.default()
val pass = Pass("solid background")
pass.addShader(shader)

pass.setClearColor([0.1, 0.2, 0.3, 1.0])

renderer.render(pass, target)