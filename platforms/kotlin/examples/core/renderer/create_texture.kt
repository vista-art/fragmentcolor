import org.fragmentcolor.*
val image = java.io.File("examples/assets/logo.png").readBytes()
val renderer = Renderer()
val tex = renderer.createTexture(image)