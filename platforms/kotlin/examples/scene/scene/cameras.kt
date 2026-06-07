import org.fragmentcolor.*

val scene = Scene.load("path/to/model.glb")

// Animate every camera the glTF shipped per frame instead of supplying
// our own. Most scenes carry a single camera, so the loop body usually
// runs once.
for (camera in scene.cameras()) {
    camera.lookAt(listOf(0.0f, 1.5f, 4.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
    camera.setAspect(16.0f / 9.0f)
}