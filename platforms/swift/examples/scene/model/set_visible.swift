import FragmentColor

let mesh = Mesh()
mesh.addVertices([
    try Vertex.pbr([0.0, 0.5, 0.0]),
    try Vertex.pbr([-0.5, -0.5, 0.0]),
    try Vertex.pbr([0.5, -0.5, 0.0]),
])
let detail = Model(mesh, Material.pbr())

// Far from the camera, hide the detail Model.
detail.setVisible(false)
// Closer again, show it.
detail.setVisible(true)