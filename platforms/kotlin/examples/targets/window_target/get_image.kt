
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(16, 16))
renderer.render(Shader.default(), target)

val image = target.getImage()