import FragmentColor

let renderer = Renderer()
let mesh = Mesh()
try mesh.addVertex(
    try Vertex([0.0, 0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.5, 1.0]),
)

let model = try await Model(mesh, Material.pbr(renderer))
try model.mesh().addVertex(
    try Vertex([-0.5, -0.5, 0.0]).set(Vertex.nORMAL, [0.0, 0.0, 1.0]).set(Vertex.uV0, [0.0, 0.0]),
)