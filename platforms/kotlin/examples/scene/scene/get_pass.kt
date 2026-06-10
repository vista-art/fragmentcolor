import org.fragmentcolor.*

val scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

// Fetch the second pass (index 1) to reconfigure it. An out-of-range
// index returns null instead.
val geometry = scene.getPass(1u)