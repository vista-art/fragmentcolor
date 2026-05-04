import org.fragmentcolor.*

val r = Renderer()
val target = r.createTextureTarget(8u, 8u)
val shader = Shader.default()
r.render(shader, target)
r.wait()
val _bytes = target.getImage()