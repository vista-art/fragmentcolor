import org.fragmentcolor.*

val scene = Scene()
val backdrop = Pass("backdrop")
val overlay = Pass("overlay")
scene.addPass(backdrop)
scene.addPass(overlay)

// Drop the backdrop; the overlay stays.
val removed = scene.removePass(backdrop)