
import org.fragmentcolor.*
val renderer = Renderer()

// Create an offscreen texture target with a size of 64x64 pixels.
val target = renderer.createTextureTarget([64, 64])

renderer.render(Shader(""), target)

// get the rendered image
val image = target.getImage()