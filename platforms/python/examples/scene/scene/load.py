from fragmentcolor import Scene, SceneSource

# File path — covers both `.gltf` JSON (with external buffers/images)
# and `.glb` binary containers.
scene = Scene.load(SceneSource.gltf("path/to/model.gltf"))

# In-memory `.glb` bytes — fetched from disk, network, or a BIN chunk
# in another format.
glb_bytes = [/* … */]
scene2 = Scene.load(SceneSource.gltf(glb_bytes))