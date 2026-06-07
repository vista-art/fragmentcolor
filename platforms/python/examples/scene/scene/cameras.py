from fragmentcolor import Scene

scene = Scene.load("path/to/model.glb")

# Animate every camera the glTF shipped per frame instead of supplying
# our own. Most scenes carry a single camera, so the loop body usually
# runs once.
for camera in scene.cameras():
    camera.look_at([0.0, 1.5, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    camera.set_aspect(16.0 / 9.0)