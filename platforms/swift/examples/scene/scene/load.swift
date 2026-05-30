import FragmentColor

// A path — """.gltf""" JSON (with external buffers/images) or a """.glb""" container.
let scene = Scene.load("path/to/model.gltf")

// In-memory """.glb""" bytes — from disk, the network, or another asset pipeline.
let glb_bytes = "/healthcheck/public/favicon.png"
let scene2 = Scene.load(glb_bytes)