from fragmentcolor import Scene
glb_bytes = open("path/to/model.glb", "rb").read()

# A path to a `.gltf` file with its buffers and images, or to a `.glb` file.
scene = Scene.load("path/to/model.gltf")

# The bytes of a `.glb` file, read from disk or the network.
scene2 = Scene.load(glb_bytes)