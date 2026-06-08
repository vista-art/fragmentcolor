from fragmentcolor import Pass, Scene

scene = Scene()
scene.add_pass(Pass("scratch"))

# Swap in a deliberate order: shadow map, then geometry, then overlay.
scene.set_passes([
    Pass("shadow"),
    Pass("geometry"),
    Pass("overlay"),
])