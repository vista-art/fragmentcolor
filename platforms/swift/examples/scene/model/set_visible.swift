import FragmentColor

let mesh = Mesh()
try mesh.addVertex(Vertex.pbr([0.0, 0.5, 0.0]))
let blob = Model(mesh, Material.pbr())

// Wide zoom level — skip the detail blobs.
blob.setVisible(false)
// Zoom back in — turn them on again.
blob.setVisible(true)