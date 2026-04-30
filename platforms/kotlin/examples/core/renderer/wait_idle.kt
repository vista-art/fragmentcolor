import org.fragmentcolor.*

val r = Renderer()
val target = r.createTextureTarget(arrayOf(8, 8))
val shader = Shader.default()
r.render(shader, target)
r.waitIdle()
val _bytes = target.getImage()