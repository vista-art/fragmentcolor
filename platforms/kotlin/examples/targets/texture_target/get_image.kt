import org.fragmentcolor.*


val renderer = Renderer()
val target = renderer.createTextureTarget(16u, 16u)
renderer.render(Shader(""), target)

val image = target.getImage().await