
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(10u, 10u)
val shader = Shader.default()

renderer.render(shader, target)