import org.fragmentcolor.*

val scene = Scene.load(SceneSource.gltf("path/to/model.glb"))

// glTF shipped a camera — animate it per frame instead of supplying our own.
if let Some(camera) = scene.cameras().intoIter().next() {
    camera.lookAt(listOf(0.0f, 1.5f, 4.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
    camera.setAspect(16.0 / 9.0)
}