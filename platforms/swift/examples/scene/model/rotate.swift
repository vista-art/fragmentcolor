import FragmentColor

let renderer = Renderer()
let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.0, 0.0]).set(Vertex.NORMAL, [0.0, 1.0, 0.0]).set(Vertex.UV0, [0.0, 0.0])
)

let model = Model(mesh, Material.pbr())
try model.rotate([0.0, 1.0, 0.0], 1.571) // 90° around Y