
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val shader = Shader.default()
renderer.render(shader, target)

val image = target.getImage().await