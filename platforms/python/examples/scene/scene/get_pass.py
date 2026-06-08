from fragmentcolor import Pass, Scene

scene = Scene()
scene.add_pass(Pass("backdrop"))
scene.add_pass(Pass("geometry"))

# Fetch the second pass (index 1) to reconfigure it. An out-of-range
# index returns None instead.
geometry = scene.get_pass(1)