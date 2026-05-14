import org.fragmentcolor.*

// File path — covers both """.gltf""" JSON (with external buffers/images)
// and """.glb""" binary containers.
val scene = Scene.load(SceneSource.gltf("path/to/model.gltf"))

// In-memory """.glb""" bytes — fetched from disk, network, or a BIN chunk
// in another format.
val glb_bytes = [/* … */]
val scene2 = Scene.load(SceneSource.gltf(glb_bytes))