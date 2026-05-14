import FragmentColor

// File path — covers both """.gltf""" JSON (with external buffers/images)
// and """.glb""" binary containers.
let scene = Scene.load(SceneSource.gltf("path/to/model.gltf"))

// In-memory """.glb""" bytes — fetched from disk, network, or a BIN chunk
// in another format.
let glb_bytes = [/* … */]
let scene2 = Scene.load(SceneSource.gltf(glb_bytes))