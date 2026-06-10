from fragmentcolor import Pass, Scene

scene = Scene()
backdrop = Pass("backdrop")
overlay = Pass("overlay")
scene.add_pass(backdrop)
scene.add_pass(overlay)

# Drop the backdrop; the overlay stays.
removed = scene.remove_pass(backdrop)