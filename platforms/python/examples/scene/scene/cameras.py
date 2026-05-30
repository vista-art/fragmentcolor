from fragmentcolor import Scene

scene = Scene.load("path/to/model.glb")

# glTF shipped a camera — animate it per frame instead of supplying our own.
if let Some(camera) = scene.cameras().into_iter().next() {
    camera.look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    camera.set_aspect(16.0 / 9.0)
}