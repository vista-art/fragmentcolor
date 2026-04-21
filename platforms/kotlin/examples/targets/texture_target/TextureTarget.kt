
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([64, 64])

val shader = Shader.default()
renderer.render(shader, target)

val image = target.getImage()