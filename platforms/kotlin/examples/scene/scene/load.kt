import org.fragmentcolor.*

// A path — """.gltf""" JSON (with external buffers/images) or a """.glb""" container.
val scene = Scene.load("path/to/model.gltf")

// In-memory """.glb""" bytes — from disk, the network, or another asset pipeline.
val glb_bytes = "/healthcheck/public/favicon.png"
val scene2 = Scene.load(glb_bytes)