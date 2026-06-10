import org.fragmentcolor.*

val scene = Scene()
scene.addPass(Pass("scratch"))

// Swap in a deliberate order: shadow map, then geometry, then overlay.
scene.setPasses(arrayOf(Pass("shadow"), Pass("geometry"), Pass("overlay"),))