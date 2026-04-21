
import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget([100, 100])

val pass1 = Pass("first")
val pass2 = Pass("second")

val frame = Frame()
frame.addPass(pass1)
frame.addPass(pass2)