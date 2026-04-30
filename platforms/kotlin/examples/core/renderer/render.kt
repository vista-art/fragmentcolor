
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(arrayOf(10, 10))
val shader = Shader.default()

renderer.render(shader, target)