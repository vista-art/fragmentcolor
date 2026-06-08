import org.fragmentcolor.*

val scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

val second = scene.getPass(1).expect("two passes were added")
second.loadPrevious()