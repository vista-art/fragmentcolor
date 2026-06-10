from fragmentcolor import Pass, Scene

scene = Scene()
scene.add_pass(Pass("backdrop"))
scene.add_pass(Pass("geometry"))

# Look the geometry pass up by name to reconfigure it. A name with no
# match returns None instead.
geometry = scene.find_pass("geometry")