import FragmentColor

let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([0.0, 0.5, 0.0]))
let blob = Model(mesh, Material.pbr())

// At a wide zoom level, skip the detail blobs.
blob.setVisible(false)
// Zoomed back in, turn them on again.
blob.setVisible(true)