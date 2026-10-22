import org.fragmentcolor.*
val glb_bytes = java.io.File("path/to/model.glb").readBytes()

// A path to a """.gltf""" file with its buffers and images, or to a """.glb""" file.
val scene = Scene.load("path/to/model.gltf")

// The bytes of a """.glb""" file, read from disk or the network.
val scene2 = Scene.load(glb_bytes)