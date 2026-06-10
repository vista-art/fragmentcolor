import org.fragmentcolor.*

val scene = Scene()
scene.addPass(Pass("backdrop"))
scene.addPass(Pass("geometry"))

// Look the geometry pass up by name to reconfigure it. A name with no
// match returns null instead.
val geometry = scene.findPass("geometry")