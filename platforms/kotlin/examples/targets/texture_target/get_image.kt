import org.fragmentcolor.*


val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(16u, 16u))
renderer.render(Shader.default(), target)

val image = target.getImage()