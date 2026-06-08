from fragmentcolor import Pass, Scene

scene = Scene()
scene.add_pass(Pass("backdrop"))
scene.add_pass(Pass("geometry"))

second = scene.get_pass(1).expect("two passes were added")
second.load_previous()