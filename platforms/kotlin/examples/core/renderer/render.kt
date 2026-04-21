
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([10, 10])
val shader = Shader.default()

renderer.render(shader, target)