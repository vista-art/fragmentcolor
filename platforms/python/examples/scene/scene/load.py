from fragmentcolor import Scene
glb_bytes = open("path/to/model.glb", "rb").read()

# A path — `.gltf` JSON (with external buffers/images) or a `.glb` container.
scene = Scene.load("path/to/model.gltf")

# In-memory `.glb` bytes — fetched from disk, the network, or another
# asset pipeline before this point.
scene2 = Scene.load(glb_bytes)