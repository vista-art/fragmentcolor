
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(64u, 64u)

val shader = Shader.default()
val pass = Pass("clipped")
pass.addShader(shader)

pass.setViewport(ScreenRegion(minX=0u, minY=0u, maxX=32u, maxY=32u))

renderer.render(pass, target)