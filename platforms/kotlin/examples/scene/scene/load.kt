import org.fragmentcolor.*
val glb_bytes = "path/to/model.glb"

// A path — """.gltf""" JSON (with external buffers/images) or a """.glb""" container.
val scene = Scene.load("path/to/model.gltf")

val bytes: ByteArray = byteArrayOf()
// In-memory """.glb""" bytes — fetched from disk, the network, or another
// asset pipeline before this point.
val scene2 = Scene.load(glb_bytes)