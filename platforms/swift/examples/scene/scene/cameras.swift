import FragmentColor

let scene = try await Scene.load("path/to/model.glb")

// Animate every camera the glTF shipped per frame instead of supplying
// our own. Most scenes carry a single camera, so the loop body usually
// runs once.
for camera in scene.cameras() {
    try camera.lookAt([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    camera.setAspect(16.0 / 9.0)
}