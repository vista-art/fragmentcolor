import FragmentColor

// A path — """.gltf""" JSON (with external buffers/images) or a """.glb""" container.
let scene = try Scene.load("path/to/model.gltf")

// In-memory """.glb""" bytes — fetched from disk, the network, or another
// asset pipeline before this point.
let glb_bytes = "/healthcheck/public/favicon.png"
let scene2 = try Scene.load(glb_bytes)