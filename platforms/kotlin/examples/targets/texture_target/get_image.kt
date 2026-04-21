import org.fragmentcolor.*


val renderer = Renderer()
val target = renderer.createTextureTarget([16, 16])
renderer.render(Shader(""), target)

val image = target.getImage()