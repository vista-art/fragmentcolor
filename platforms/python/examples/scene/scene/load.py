from fragmentcolor import Scene

# A path — `.gltf` JSON (with external buffers/images) or a `.glb` container.
scene = Scene.load("path/to/model.gltf")

# In-memory `.glb` bytes — from disk, the network, or another asset pipeline.
glb_bytes = open("path/to/model.glb", "rb").read()
scene2 = Scene.load(glb_bytes)